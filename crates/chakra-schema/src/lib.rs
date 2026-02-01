//! Schema introspection and DDL generation for Chakra ORM
//!
//! This crate provides:
//! - Schema introspection from databases
//! - DDL generation for schema changes
//! - Schema comparison and diff generation
//! - Database-agnostic schema representation

pub mod ddl;
pub mod diff;
pub mod introspect;
pub mod schema;

pub use ddl::{DdlGenerator, DdlStatement};
pub use diff::{SchemaDiff, SchemaDiffer};
pub use introspect::SchemaIntrospector;
pub use schema::{
    Column, Constraint, ConstraintType, ForeignKey, Index, Schema, Table,
};
