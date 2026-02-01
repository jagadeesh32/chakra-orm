//! Migration engine for Chakra ORM
//!
//! This crate provides:
//! - Migration file management
//! - Schema change detection
//! - Migration execution
//! - Rollback support
//! - Django-style auto migrations

pub mod executor;
pub mod file;
pub mod generator;
pub mod history;
pub mod migration;
pub mod planner;

pub use executor::MigrationExecutor;
pub use file::{MigrationFile, MigrationLoader};
pub use generator::MigrationGenerator;
pub use history::{MigrationHistory, MigrationRecord};
pub use migration::{Migration, MigrationDirection, MigrationStatus};
pub use planner::MigrationPlanner;
