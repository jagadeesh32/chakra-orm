//! Model system for Chakra ORM
//!
//! This module provides:
//! - `Model` trait for ORM models
//! - `ModelMeta` for model metadata
//! - `FieldMeta` for field metadata
//! - `Related` for relationship handling

use crate::error::{ChakraError, ModelError, Result};
use crate::result::Row;
use crate::types::{FieldType, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global model registry
static MODEL_REGISTRY: RwLock<Option<ModelRegistry>> = RwLock::new(None);

/// Trait for ORM models
pub trait Model: Sized + Send + Sync {
    /// The primary key type
    type PrimaryKey: Clone + Send + Sync + Into<Value>;

    /// Get the table name
    fn table_name() -> &'static str;

    /// Get model metadata
    fn meta() -> &'static ModelMeta;

    /// Get field metadata
    fn fields() -> &'static [FieldMeta];

    /// Get the primary key value
    fn primary_key(&self) -> &Self::PrimaryKey;

    /// Create from a database row
    fn from_row(row: &Row) -> Result<Self>;

    /// Convert to a map of values
    fn to_values(&self) -> HashMap<String, Value>;

    /// Get a field value by name
    fn get_field(&self, name: &str) -> Option<Value>;

    /// Set a field value by name
    fn set_field(&mut self, name: &str, value: Value) -> Result<()>;
}

/// Metadata for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMeta {
    /// Model name (struct/class name)
    pub name: String,
    /// Table name
    pub table: String,
    /// Schema name (optional)
    pub schema: Option<String>,
    /// Primary key field(s)
    pub primary_key: Vec<String>,
    /// Field metadata
    pub fields: Vec<FieldMeta>,
    /// Index definitions
    pub indexes: Vec<IndexMeta>,
    /// Constraint definitions
    pub constraints: Vec<ConstraintMeta>,
    /// Relationship metadata
    pub relationships: Vec<RelationMeta>,
}

impl ModelMeta {
    /// Create a new ModelMeta builder
    pub fn builder(name: impl Into<String>, table: impl Into<String>) -> ModelMetaBuilder {
        ModelMetaBuilder::new(name, table)
    }

    /// Get a field by name
    pub fn get_field(&self, name: &str) -> Option<&FieldMeta> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get the primary key field(s)
    pub fn primary_key_fields(&self) -> Vec<&FieldMeta> {
        self.fields
            .iter()
            .filter(|f| f.primary_key)
            .collect()
    }
}

/// Builder for ModelMeta
pub struct ModelMetaBuilder {
    meta: ModelMeta,
}

impl ModelMetaBuilder {
    pub fn new(name: impl Into<String>, table: impl Into<String>) -> Self {
        Self {
            meta: ModelMeta {
                name: name.into(),
                table: table.into(),
                schema: None,
                primary_key: Vec::new(),
                fields: Vec::new(),
                indexes: Vec::new(),
                constraints: Vec::new(),
                relationships: Vec::new(),
            },
        }
    }

    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.meta.schema = Some(schema.into());
        self
    }

    pub fn field(mut self, field: FieldMeta) -> Self {
        if field.primary_key {
            self.meta.primary_key.push(field.name.clone());
        }
        self.meta.fields.push(field);
        self
    }

    pub fn index(mut self, index: IndexMeta) -> Self {
        self.meta.indexes.push(index);
        self
    }

    pub fn constraint(mut self, constraint: ConstraintMeta) -> Self {
        self.meta.constraints.push(constraint);
        self
    }

    pub fn relationship(mut self, rel: RelationMeta) -> Self {
        self.meta.relationships.push(rel);
        self
    }

    pub fn build(self) -> ModelMeta {
        self.meta
    }
}

/// Metadata for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMeta {
    /// Field name
    pub name: String,
    /// Column name (if different from field name)
    pub column: Option<String>,
    /// Field type
    pub field_type: FieldType,
    /// Is this the primary key?
    pub primary_key: bool,
    /// Auto-increment?
    pub auto_increment: bool,
    /// Allow null?
    pub nullable: bool,
    /// Has unique constraint?
    pub unique: bool,
    /// Has index?
    pub index: bool,
    /// Default value
    pub default: Option<FieldDefault>,
    /// Foreign key reference
    pub foreign_key: Option<ForeignKeyMeta>,
}

impl FieldMeta {
    /// Create a new FieldMeta builder
    pub fn builder(name: impl Into<String>, field_type: FieldType) -> FieldMetaBuilder {
        FieldMetaBuilder::new(name, field_type)
    }

    /// Get the column name (field name if not specified)
    pub fn column_name(&self) -> &str {
        self.column.as_deref().unwrap_or(&self.name)
    }
}

/// Builder for FieldMeta
pub struct FieldMetaBuilder {
    meta: FieldMeta,
}

impl FieldMetaBuilder {
    pub fn new(name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            meta: FieldMeta {
                name: name.into(),
                column: None,
                field_type,
                primary_key: false,
                auto_increment: false,
                nullable: false,
                unique: false,
                index: false,
                default: None,
                foreign_key: None,
            },
        }
    }

    pub fn column(mut self, column: impl Into<String>) -> Self {
        self.meta.column = Some(column.into());
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.meta.primary_key = true;
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.meta.auto_increment = true;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.meta.nullable = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.meta.unique = true;
        self
    }

    pub fn index(mut self) -> Self {
        self.meta.index = true;
        self
    }

    pub fn default(mut self, default: FieldDefault) -> Self {
        self.meta.default = Some(default);
        self
    }

    pub fn default_value(mut self, value: Value) -> Self {
        self.meta.default = Some(FieldDefault::Value(value));
        self
    }

    pub fn default_expr(mut self, expr: impl Into<String>) -> Self {
        self.meta.default = Some(FieldDefault::Expression(expr.into()));
        self
    }

    pub fn foreign_key(mut self, fk: ForeignKeyMeta) -> Self {
        self.meta.foreign_key = Some(fk);
        self
    }

    pub fn build(self) -> FieldMeta {
        self.meta
    }
}

/// Default value for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldDefault {
    /// Static value
    Value(Value),
    /// SQL expression (e.g., "now()")
    Expression(String),
    /// Auto-increment
    AutoIncrement,
    /// Generate UUID
    Uuid,
}

/// Foreign key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyMeta {
    /// Referenced table
    pub table: String,
    /// Referenced column
    pub column: String,
    /// On delete action
    pub on_delete: ForeignKeyAction,
    /// On update action
    pub on_update: ForeignKeyAction,
}

/// Foreign key action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    NoAction,
    Cascade,
    SetNull,
    SetDefault,
    Restrict,
}

impl ForeignKeyAction {
    pub fn as_sql(&self) -> &'static str {
        match self {
            ForeignKeyAction::NoAction => "NO ACTION",
            ForeignKeyAction::Cascade => "CASCADE",
            ForeignKeyAction::SetNull => "SET NULL",
            ForeignKeyAction::SetDefault => "SET DEFAULT",
            ForeignKeyAction::Restrict => "RESTRICT",
        }
    }
}

/// Index metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMeta {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub descending: bool,
    pub where_clause: Option<String>,
}

impl IndexMeta {
    pub fn new(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            unique: false,
            descending: false,
            where_clause: None,
        }
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn descending(mut self) -> Self {
        self.descending = true;
        self
    }

    pub fn where_clause(mut self, clause: impl Into<String>) -> Self {
        self.where_clause = Some(clause.into());
        self
    }
}

/// Constraint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintMeta {
    Unique {
        name: String,
        columns: Vec<String>,
    },
    Check {
        name: String,
        expression: String,
    },
    ForeignKey {
        name: String,
        columns: Vec<String>,
        references_table: String,
        references_columns: Vec<String>,
        on_delete: ForeignKeyAction,
        on_update: ForeignKeyAction,
    },
}

/// Relationship metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationMeta {
    pub name: String,
    pub relation_type: RelationType,
    pub target_model: String,
    pub foreign_key: Option<String>,
    pub through_table: Option<String>,
    pub back_populates: Option<String>,
}

/// Relationship type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Wrapper for lazy-loaded relationships
#[derive(Debug)]
pub struct Related<T> {
    value: Option<T>,
    loaded: bool,
}

impl<T> Related<T> {
    /// Create a new unloaded relationship
    pub fn new() -> Self {
        Self {
            value: None,
            loaded: false,
        }
    }

    /// Create a loaded relationship
    pub fn loaded(value: T) -> Self {
        Self {
            value: Some(value),
            loaded: true,
        }
    }

    /// Check if loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get the value if loaded
    pub fn get(&self) -> Result<&T> {
        if self.loaded {
            self.value.as_ref().ok_or_else(|| {
                ChakraError::Model(ModelError::RelationshipNotLoaded {
                    relationship: "unknown".to_string(),
                })
            })
        } else {
            Err(ChakraError::Model(ModelError::RelationshipNotLoaded {
                relationship: "unknown".to_string(),
            }))
        }
    }

    /// Set the value
    pub fn set(&mut self, value: T) {
        self.value = Some(value);
        self.loaded = true;
    }

    /// Take the value
    pub fn take(&mut self) -> Option<T> {
        self.loaded = false;
        self.value.take()
    }
}

impl<T> Default for Related<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for Related<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            loaded: self.loaded,
        }
    }
}

/// Model registry for runtime model lookup
#[derive(Debug, Default)]
pub struct ModelRegistry {
    models: HashMap<String, Arc<ModelMeta>>,
}

impl ModelRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a model
    pub fn register(&mut self, meta: ModelMeta) {
        self.models.insert(meta.name.clone(), Arc::new(meta));
    }

    /// Get a model by name
    pub fn get(&self, name: &str) -> Option<Arc<ModelMeta>> {
        self.models.get(name).cloned()
    }

    /// Get all registered models
    pub fn all(&self) -> impl Iterator<Item = &Arc<ModelMeta>> {
        self.models.values()
    }
}

/// Initialize the global model registry
pub fn init_registry() {
    let mut lock = MODEL_REGISTRY.write().unwrap();
    if lock.is_none() {
        *lock = Some(ModelRegistry::new());
    }
}

/// Register a model in the global registry
pub fn register_model(meta: ModelMeta) {
    init_registry();
    let mut lock = MODEL_REGISTRY.write().unwrap();
    if let Some(registry) = lock.as_mut() {
        registry.register(meta);
    }
}

/// Get a model from the global registry
pub fn get_model(name: &str) -> Option<Arc<ModelMeta>> {
    let lock = MODEL_REGISTRY.read().unwrap();
    lock.as_ref().and_then(|r| r.get(name))
}

/// Placeholder for Field descriptor used in Python-style model definitions
#[derive(Debug, Clone)]
pub struct Field {
    pub meta: FieldMeta,
}

impl Field {
    pub fn new(field_type: FieldType) -> Self {
        Self {
            meta: FieldMeta {
                name: String::new(),
                column: None,
                field_type,
                primary_key: false,
                auto_increment: false,
                nullable: false,
                unique: false,
                index: false,
                default: None,
                foreign_key: None,
            },
        }
    }

    pub fn primary_key(mut self) -> Self {
        self.meta.primary_key = true;
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.meta.auto_increment = true;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.meta.nullable = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.meta.unique = true;
        self
    }

    pub fn index(mut self) -> Self {
        self.meta.index = true;
        self
    }

    pub fn default(mut self, value: Value) -> Self {
        self.meta.default = Some(FieldDefault::Value(value));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_meta_builder() {
        let meta = ModelMeta::builder("User", "users")
            .schema("public")
            .field(
                FieldMeta::builder("id", FieldType::BigInt)
                    .primary_key()
                    .auto_increment()
                    .build(),
            )
            .field(
                FieldMeta::builder("name", FieldType::string(100))
                    .build(),
            )
            .build();

        assert_eq!(meta.name, "User");
        assert_eq!(meta.table, "users");
        assert_eq!(meta.schema, Some("public".to_string()));
        assert_eq!(meta.fields.len(), 2);
        assert_eq!(meta.primary_key, vec!["id"]);
    }

    #[test]
    fn test_related() {
        let mut rel: Related<Vec<i32>> = Related::new();
        assert!(!rel.is_loaded());

        rel.set(vec![1, 2, 3]);
        assert!(rel.is_loaded());
        assert_eq!(rel.get().unwrap(), &vec![1, 2, 3]);
    }
}
