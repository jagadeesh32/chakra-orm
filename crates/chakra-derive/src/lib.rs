//! Procedural macros for Chakra ORM
//!
//! This crate provides derive macros for:
//! - `#[derive(Model)]` - Derive the Model trait
//! - `#[derive(FromRow)]` - Derive row deserialization
//! - `#[derive(IntoParams)]` - Derive parameter conversion

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod field;
mod model;
mod from_row;

/// Derive the Model trait for a struct
///
/// # Example
///
/// ```ignore
/// use chakra_derive::Model;
///
/// #[derive(Model)]
/// #[chakra(table = "users")]
/// struct User {
///     #[chakra(primary_key, auto_increment)]
///     id: i64,
///
///     #[chakra(column = "user_name")]
///     name: String,
///
///     #[chakra(nullable)]
///     email: Option<String>,
///
///     #[chakra(default = "now()")]
///     created_at: chrono::DateTime<chrono::Utc>,
/// }
/// ```
#[proc_macro_derive(Model, attributes(chakra))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match model::expand_model(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive the FromRow trait for a struct
///
/// # Example
///
/// ```ignore
/// use chakra_derive::FromRow;
///
/// #[derive(FromRow)]
/// struct UserRow {
///     id: i64,
///     name: String,
///     email: Option<String>,
/// }
/// ```
#[proc_macro_derive(FromRow, attributes(chakra))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match from_row::expand_from_row(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Attribute macro for defining a model inline
///
/// # Example
///
/// ```ignore
/// use chakra_derive::model;
///
/// #[model(table = "users", schema = "public")]
/// struct User {
///     #[primary_key]
///     id: i64,
///     name: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = parse_macro_input!(item as DeriveInput);

    match model::expand_model_attribute(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
