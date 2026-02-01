//! # Chakra Core
//!
//! The core query engine for Chakra ORM. This crate provides:
//!
//! - Query building and SQL generation
//! - Type system and field definitions
//! - Expression evaluation (F, Q objects)
//! - Result mapping and decoding
//! - Model metadata and registry
//!
//! ## Example
//!
//! ```rust,ignore
//! use chakra_core::prelude::*;
//!
//! let query = Query::select()
//!     .from("users")
//!     .columns(&["id", "name", "email"])
//!     .filter(Expr::eq("is_active", true))
//!     .order_by("created_at", Order::Desc)
//!     .limit(10)
//!     .build();
//!
//! let sql = PostgresDialect.generate(&query);
//! ```

pub mod error;
pub mod expr;
pub mod model;
pub mod query;
pub mod result;
pub mod sql;
pub mod types;

// Re-export derive macros if enabled
#[cfg(feature = "derive")]
pub use chakra_derive::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{ChakraError, Result};
    pub use crate::expr::{Expr, F, Q};
    pub use crate::model::{Field, FieldMeta, Model, ModelMeta, Related};
    pub use crate::query::{Order, Query, QueryBuilder};
    pub use crate::result::{FromRow, Row, RowStream};
    pub use crate::sql::{Dialect, PostgresDialect, SqlFragment};
    pub use crate::types::{FieldType, Value};

    #[cfg(feature = "derive")]
    pub use chakra_derive::Model;
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
