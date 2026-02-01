//! FromRow derive macro implementation

use crate::field::FieldAttrs;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident};

/// Container attributes for FromRow
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(chakra), supports(struct_named))]
struct FromRowAttrs {
    ident: Ident,
    data: darling::ast::Data<(), FieldAttrs>,
}

impl FromRowAttrs {
    fn fields(&self) -> Vec<&FieldAttrs> {
        match &self.data {
            darling::ast::Data::Struct(fields) => fields.iter().filter(|f| !f.skip).collect(),
            _ => vec![],
        }
    }
}

/// Expand the FromRow derive macro
pub fn expand_from_row(input: DeriveInput) -> syn::Result<TokenStream> {
    let attrs = FromRowAttrs::from_derive_input(&input)?;

    let struct_name = &attrs.ident;
    let fields = attrs.fields();

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

    let expanded = quote! {
        impl chakra_core::result::FromRow for #struct_name {
            fn from_row(row: &chakra_core::result::Row) -> chakra_core::error::Result<Self> {
                Ok(Self {
                    #(#from_row_fields),*
                })
            }
        }
    };

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    // Tests would go here
}
