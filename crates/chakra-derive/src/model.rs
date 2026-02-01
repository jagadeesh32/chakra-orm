//! Model derive macro implementation

use crate::field::FieldAttrs;
use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

/// Container-level attributes for Model
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(chakra), supports(struct_named))]
struct ModelAttrs {
    /// Struct identifier
    ident: Ident,
    /// Struct data
    data: darling::ast::Data<(), FieldAttrs>,

    /// Table name override
    #[darling(default)]
    table: Option<String>,

    /// Schema name
    #[darling(default)]
    schema: Option<String>,

    /// Rename all fields strategy
    #[darling(default)]
    rename_all: Option<String>,
}

impl ModelAttrs {
    /// Get the table name
    fn table_name(&self) -> String {
        self.table
            .clone()
            .unwrap_or_else(|| self.ident.to_string().to_case(Case::Snake) + "s")
    }

    /// Get all fields
    fn fields(&self) -> Vec<&FieldAttrs> {
        match &self.data {
            darling::ast::Data::Struct(fields) => fields.iter().filter(|f| !f.skip).collect(),
            _ => vec![],
        }
    }

    /// Get primary key fields
    fn primary_key_fields(&self) -> Vec<&FieldAttrs> {
        self.fields().into_iter().filter(|f| f.primary_key).collect()
    }
}

/// Expand the Model derive macro
pub fn expand_model(input: DeriveInput) -> syn::Result<TokenStream> {
    let attrs = ModelAttrs::from_derive_input(&input)?;

    let struct_name = &attrs.ident;
    let table_name = attrs.table_name();
    let schema = match &attrs.schema {
        Some(s) => quote! { Some(#s.to_string()) },
        None => quote! { None },
    };

    let fields = attrs.fields();
    let pk_fields = attrs.primary_key_fields();

    // Determine primary key type
    let pk_type = if pk_fields.len() == 1 {
        let pk = pk_fields[0];
        let ty = &pk.ty;
        quote! { #ty }
    } else if pk_fields.is_empty() {
        // Default to i64
        quote! { i64 }
    } else {
        // Composite key - use tuple
        let types: Vec<_> = pk_fields.iter().map(|f| &f.ty).collect();
        quote! { (#(#types),*) }
    };

    // Generate field metadata
    let field_metas: Vec<_> = fields.iter().map(|f| f.to_field_meta()).collect();

    // Generate primary_key() method
    let pk_impl = if pk_fields.len() == 1 {
        let pk_name = pk_fields[0].field_name();
        quote! {
            fn primary_key(&self) -> &Self::PrimaryKey {
                &self.#pk_name
            }
        }
    } else if pk_fields.is_empty() {
        quote! {
            fn primary_key(&self) -> &Self::PrimaryKey {
                &0i64
            }
        }
    } else {
        let pk_names: Vec<_> = pk_fields.iter().map(|f| f.field_name()).collect();
        quote! {
            fn primary_key(&self) -> &Self::PrimaryKey {
                &(#(self.#pk_names.clone()),*)
            }
        }
    };

    // Generate from_row() method
    let from_row_fields: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = f.field_name();
            let col_name = f.column_name();
            if f.is_option() {
                quote! {
                    #field_name: row.try_get(#col_name)?
                }
            } else {
                quote! {
                    #field_name: row.get_as(#col_name)?
                }
            }
        })
        .collect();

    // Generate to_values() method
    let to_values_fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.auto_increment) // Skip auto-increment on insert
        .map(|f| {
            let field_name = f.field_name();
            let col_name = f.column_name();
            quote! {
                map.insert(#col_name.to_string(), (&self.#field_name).into());
            }
        })
        .collect();

    // Generate get_field() method
    let get_field_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = f.field_name();
            let col_name = f.column_name();
            quote! {
                #col_name => Some((&self.#field_name).into())
            }
        })
        .collect();

    // Generate set_field() method
    let set_field_arms: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = f.field_name();
            let col_name = f.column_name();
            let ty = &f.ty;
            quote! {
                #col_name => {
                    self.#field_name = chakra_core::result::FromValue::from_value(&value)?;
                    Ok(())
                }
            }
        })
        .collect();

    // Primary key column names
    let pk_columns: Vec<_> = pk_fields.iter().map(|f| f.column_name()).collect();

    // Static metadata
    let model_meta_name = Ident::new(
        &format!("{}_META", struct_name.to_string().to_uppercase()),
        struct_name.span(),
    );
    let fields_name = Ident::new(
        &format!("{}_FIELDS", struct_name.to_string().to_uppercase()),
        struct_name.span(),
    );
    let fields_len = field_metas.len();

    let expanded = quote! {
        // Static metadata
        static #model_meta_name: std::sync::OnceLock<chakra_core::model::ModelMeta> = std::sync::OnceLock::new();
        static #fields_name: std::sync::OnceLock<[chakra_core::model::FieldMeta; #fields_len]> = std::sync::OnceLock::new();

        impl chakra_core::model::Model for #struct_name {
            type PrimaryKey = #pk_type;

            fn table_name() -> &'static str {
                #table_name
            }

            fn meta() -> &'static chakra_core::model::ModelMeta {
                #model_meta_name.get_or_init(|| {
                    chakra_core::model::ModelMeta {
                        name: stringify!(#struct_name).to_string(),
                        table: #table_name.to_string(),
                        schema: #schema,
                        primary_key: vec![#(#pk_columns.to_string()),*],
                        fields: Self::fields().to_vec(),
                        indexes: Vec::new(),
                        constraints: Vec::new(),
                        relationships: Vec::new(),
                    }
                })
            }

            fn fields() -> &'static [chakra_core::model::FieldMeta] {
                #fields_name.get_or_init(|| [
                    #(#field_metas),*
                ])
            }

            #pk_impl

            fn from_row(row: &chakra_core::result::Row) -> chakra_core::error::Result<Self> {
                Ok(Self {
                    #(#from_row_fields),*
                })
            }

            fn to_values(&self) -> std::collections::HashMap<String, chakra_core::types::Value> {
                let mut map = std::collections::HashMap::new();
                #(#to_values_fields)*
                map
            }

            fn get_field(&self, name: &str) -> Option<chakra_core::types::Value> {
                match name {
                    #(#get_field_arms,)*
                    _ => None,
                }
            }

            fn set_field(&mut self, name: &str, value: chakra_core::types::Value) -> chakra_core::error::Result<()> {
                match name {
                    #(#set_field_arms,)*
                    _ => Err(chakra_core::error::ChakraError::internal(
                        format!("Unknown field: {}", name)
                    )),
                }
            }
        }

        // Also implement FromRow
        impl chakra_core::result::FromRow for #struct_name {
            fn from_row(row: &chakra_core::result::Row) -> chakra_core::error::Result<Self> {
                <Self as chakra_core::model::Model>::from_row(row)
            }
        }
    };

    Ok(expanded)
}

/// Expand the model attribute macro
pub fn expand_model_attribute(
    _attr: TokenStream,
    input: DeriveInput,
) -> syn::Result<TokenStream> {
    // For now, just forward to the derive macro
    // In a full implementation, we could parse additional attributes here
    let derive_impl = expand_model(input.clone())?;

    let struct_def = quote! { #input };

    Ok(quote! {
        #[derive(Debug, Clone)]
        #struct_def

        #derive_impl
    })
}

#[cfg(test)]
mod tests {
    // Tests would go here
}
