//! Field parsing and metadata extraction

use darling::{FromField, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type, Visibility};

/// Field-level attributes
#[derive(Debug, FromField)]
#[darling(attributes(chakra))]
pub struct FieldAttrs {
    /// Field identifier
    pub ident: Option<Ident>,
    /// Field visibility
    pub vis: Visibility,
    /// Field type
    pub ty: Type,

    /// Column name override
    #[darling(default)]
    pub column: Option<String>,

    /// Is this the primary key?
    #[darling(default)]
    pub primary_key: bool,

    /// Is this auto-increment?
    #[darling(default)]
    pub auto_increment: bool,

    /// Is this nullable?
    #[darling(default)]
    pub nullable: bool,

    /// Is this unique?
    #[darling(default)]
    pub unique: bool,

    /// Is this indexed?
    #[darling(default)]
    pub index: bool,

    /// Default value expression
    #[darling(default)]
    pub default: Option<String>,

    /// Skip this field
    #[darling(default)]
    pub skip: bool,

    /// Foreign key reference (table.column)
    #[darling(default)]
    pub references: Option<String>,

    /// JSON field
    #[darling(default)]
    pub json: bool,

    /// Rename strategy override
    #[darling(default)]
    pub rename: Option<String>,
}

impl FieldAttrs {
    /// Get the column name for this field
    pub fn column_name(&self) -> String {
        if let Some(ref col) = self.column {
            col.clone()
        } else if let Some(ref rename) = self.rename {
            rename.clone()
        } else {
            self.ident
                .as_ref()
                .map(|i| to_snake_case(&i.to_string()))
                .unwrap_or_default()
        }
    }

    /// Get the field name
    pub fn field_name(&self) -> &Ident {
        self.ident.as_ref().expect("field must have a name")
    }

    /// Check if this is an Option type
    pub fn is_option(&self) -> bool {
        is_option_type(&self.ty)
    }

    /// Get the inner type if Option
    pub fn inner_type(&self) -> &Type {
        if let Type::Path(ref path) = self.ty {
            if let Some(segment) = path.path.segments.last() {
                if segment.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                        if let Some(syn::GenericArgument::Type(ref inner)) = args.args.first() {
                            return inner;
                        }
                    }
                }
            }
        }
        &self.ty
    }

    /// Generate FieldType expression
    pub fn field_type_expr(&self) -> TokenStream {
        let ty = self.inner_type();
        type_to_field_type(ty, self.json)
    }

    /// Generate FieldMeta construction
    pub fn to_field_meta(&self) -> TokenStream {
        let name = self.column_name();
        let field_type = self.field_type_expr();
        let primary_key = self.primary_key;
        let auto_increment = self.auto_increment;
        let nullable = self.nullable || self.is_option();
        let unique = self.unique;
        let index = self.index;

        let default_expr = if let Some(ref default) = self.default {
            quote! { Some(chakra_core::model::FieldDefault::Expression(#default.to_string())) }
        } else if self.auto_increment {
            quote! { Some(chakra_core::model::FieldDefault::AutoIncrement) }
        } else {
            quote! { None }
        };

        let fk_expr = if let Some(ref refs) = self.references {
            let parts: Vec<&str> = refs.split('.').collect();
            if parts.len() == 2 {
                let table = parts[0];
                let column = parts[1];
                quote! {
                    Some(chakra_core::model::ForeignKeyMeta {
                        table: #table.to_string(),
                        column: #column.to_string(),
                        on_delete: chakra_core::model::ForeignKeyAction::NoAction,
                        on_update: chakra_core::model::ForeignKeyAction::NoAction,
                    })
                }
            } else {
                quote! { None }
            }
        } else {
            quote! { None }
        };

        quote! {
            chakra_core::model::FieldMeta {
                name: #name.to_string(),
                column: None,
                field_type: #field_type,
                primary_key: #primary_key,
                auto_increment: #auto_increment,
                nullable: #nullable,
                unique: #unique,
                index: #index,
                default: #default_expr,
                foreign_key: #fk_expr,
            }
        }
    }
}

/// Convert a type to a FieldType expression
fn type_to_field_type(ty: &Type, is_json: bool) -> TokenStream {
    if is_json {
        return quote! { chakra_core::types::FieldType::Json };
    }

    if let Type::Path(ref path) = ty {
        if let Some(segment) = path.path.segments.last() {
            let type_name = segment.ident.to_string();
            return match type_name.as_str() {
                "i16" => quote! { chakra_core::types::FieldType::SmallInt },
                "i32" => quote! { chakra_core::types::FieldType::Integer },
                "i64" => quote! { chakra_core::types::FieldType::BigInt },
                "f32" => quote! { chakra_core::types::FieldType::Float },
                "f64" => quote! { chakra_core::types::FieldType::Double },
                "bool" => quote! { chakra_core::types::FieldType::Boolean },
                "String" => quote! { chakra_core::types::FieldType::Text },
                "Uuid" => quote! { chakra_core::types::FieldType::Uuid },
                "DateTime" => quote! { chakra_core::types::FieldType::TimestampTz },
                "NaiveDate" => quote! { chakra_core::types::FieldType::Date },
                "NaiveTime" => quote! { chakra_core::types::FieldType::Time },
                "Value" => quote! { chakra_core::types::FieldType::Json },
                "Vec" => {
                    // Check if it's Vec<u8> for bytes
                    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                        if let Some(syn::GenericArgument::Type(Type::Path(inner_path))) =
                            args.args.first()
                        {
                            if let Some(inner_seg) = inner_path.path.segments.last() {
                                if inner_seg.ident == "u8" {
                                    return quote! { chakra_core::types::FieldType::Bytes };
                                }
                            }
                        }
                    }
                    quote! { chakra_core::types::FieldType::Json }
                }
                _ => quote! { chakra_core::types::FieldType::Text },
            };
        }
    }

    quote! { chakra_core::types::FieldType::Text }
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(ref path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    use convert_case::{Case, Casing};
    s.to_case(Case::Snake)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserName"), "user_name");
        assert_eq!(to_snake_case("userName"), "user_name");
        assert_eq!(to_snake_case("user_name"), "user_name");
    }
}
