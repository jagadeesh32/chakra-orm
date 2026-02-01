//! Query builder for Chakra ORM
//!
//! This module provides a fluent API for building SQL queries.

use crate::expr::Expr;
use crate::types::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Order::Asc => "ASC",
            Order::Desc => "DESC",
        }
    }
}

/// Join type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl JoinType {
    pub fn as_sql(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL OUTER JOIN",
        }
    }
}

/// A join clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Join {
    pub join_type: JoinType,
    pub table: String,
    pub alias: Option<String>,
    pub on: Expr,
}

/// Order by clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub column: String,
    pub order: Order,
    pub nulls: Option<NullsOrder>,
}

/// Nulls ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NullsOrder {
    First,
    Last,
}

/// Query type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
}

/// A complete query representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub query_type: QueryType,
    pub table: String,
    pub alias: Option<String>,
    pub columns: Vec<String>,
    pub values: Vec<HashMap<String, Value>>,
    pub where_clause: Option<Expr>,
    pub joins: Vec<Join>,
    pub order_by: Vec<OrderBy>,
    pub group_by: Vec<String>,
    pub having: Option<Expr>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub distinct: bool,
    pub returning: Vec<String>,
    pub for_update: bool,
}

impl Query {
    /// Create a new SELECT query builder
    pub fn select() -> QueryBuilder {
        QueryBuilder::new(QueryType::Select)
    }

    /// Create a new INSERT query builder
    pub fn insert() -> QueryBuilder {
        QueryBuilder::new(QueryType::Insert)
    }

    /// Create a new UPDATE query builder
    pub fn update() -> QueryBuilder {
        QueryBuilder::new(QueryType::Update)
    }

    /// Create a new DELETE query builder
    pub fn delete() -> QueryBuilder {
        QueryBuilder::new(QueryType::Delete)
    }
}

/// Fluent query builder
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    query_type: QueryType,
    table: Option<String>,
    alias: Option<String>,
    columns: Vec<String>,
    values: Vec<HashMap<String, Value>>,
    where_clauses: Vec<Expr>,
    joins: Vec<Join>,
    order_by: Vec<OrderBy>,
    group_by: Vec<String>,
    having: Option<Expr>,
    limit: Option<usize>,
    offset: Option<usize>,
    distinct: bool,
    returning: Vec<String>,
    for_update: bool,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new(query_type: QueryType) -> Self {
        Self {
            query_type,
            table: None,
            alias: None,
            columns: Vec::new(),
            values: Vec::new(),
            where_clauses: Vec::new(),
            joins: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            having: None,
            limit: None,
            offset: None,
            distinct: false,
            returning: Vec::new(),
            for_update: false,
        }
    }

    /// Set the table name
    pub fn from(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    /// Set the table name (alias for from)
    pub fn table(self, table: impl Into<String>) -> Self {
        self.from(table)
    }

    /// Set table alias
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// Set columns to select
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a column
    pub fn column(mut self, column: impl Into<String>) -> Self {
        self.columns.push(column.into());
        self
    }

    /// Select all columns
    pub fn all_columns(mut self) -> Self {
        self.columns = vec!["*".to_string()];
        self
    }

    /// Add a WHERE filter
    pub fn filter(mut self, expr: Expr) -> Self {
        self.where_clauses.push(expr);
        self
    }

    /// Add a WHERE condition with column = value
    pub fn where_eq(self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.filter(Expr::eq(column, value))
    }

    /// Add an INNER JOIN
    pub fn join(mut self, table: impl Into<String>, on: Expr) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            table: table.into(),
            alias: None,
            on,
        });
        self
    }

    /// Add a LEFT JOIN
    pub fn left_join(mut self, table: impl Into<String>, on: Expr) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Left,
            table: table.into(),
            alias: None,
            on,
        });
        self
    }

    /// Add ORDER BY
    pub fn order_by(mut self, column: impl Into<String>, order: Order) -> Self {
        self.order_by.push(OrderBy {
            column: column.into(),
            order,
            nulls: None,
        });
        self
    }

    /// Add ORDER BY ASC
    pub fn order_by_asc(self, column: impl Into<String>) -> Self {
        self.order_by(column, Order::Asc)
    }

    /// Add ORDER BY DESC
    pub fn order_by_desc(self, column: impl Into<String>) -> Self {
        self.order_by(column, Order::Desc)
    }

    /// Add GROUP BY
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add HAVING
    pub fn having(mut self, expr: Expr) -> Self {
        self.having = Some(expr);
        self
    }

    /// Set LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set DISTINCT
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Set RETURNING columns
    pub fn returning(mut self, columns: &[&str]) -> Self {
        self.returning = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set FOR UPDATE
    pub fn for_update(mut self) -> Self {
        self.for_update = true;
        self
    }

    /// Set values for INSERT
    pub fn values(mut self, values: HashMap<String, Value>) -> Self {
        self.values.push(values);
        self
    }

    /// Set a single value
    pub fn set(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        if self.values.is_empty() {
            self.values.push(HashMap::new());
        }
        if let Some(vals) = self.values.last_mut() {
            vals.insert(column.into(), value.into());
        }
        self
    }

    /// Build the query
    pub fn build(self) -> Query {
        let where_clause = if self.where_clauses.is_empty() {
            None
        } else if self.where_clauses.len() == 1 {
            Some(self.where_clauses.into_iter().next().unwrap())
        } else {
            Some(Expr::And(self.where_clauses))
        };

        Query {
            query_type: self.query_type,
            table: self.table.unwrap_or_default(),
            alias: self.alias,
            columns: if self.columns.is_empty() {
                vec!["*".to_string()]
            } else {
                self.columns
            },
            values: self.values,
            where_clause,
            joins: self.joins,
            order_by: self.order_by,
            group_by: self.group_by,
            having: self.having,
            limit: self.limit,
            offset: self.offset,
            distinct: self.distinct,
            returning: self.returning,
            for_update: self.for_update,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_query_builder() {
        let query = Query::select()
            .from("users")
            .columns(&["id", "name", "email"])
            .filter(Expr::eq("is_active", true))
            .order_by_desc("created_at")
            .limit(10)
            .build();

        assert_eq!(query.table, "users");
        assert_eq!(query.columns, vec!["id", "name", "email"]);
        assert!(query.where_clause.is_some());
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_insert_query_builder() {
        let query = Query::insert()
            .table("users")
            .set("name", "Alice")
            .set("email", "alice@example.com")
            .returning(&["id"])
            .build();

        assert_eq!(query.query_type, QueryType::Insert);
        assert_eq!(query.table, "users");
        assert_eq!(query.values.len(), 1);
        assert_eq!(query.returning, vec!["id"]);
    }

    #[test]
    fn test_update_query_builder() {
        let query = Query::update()
            .table("users")
            .set("name", "Alice Updated")
            .filter(Expr::eq("id", 1))
            .build();

        assert_eq!(query.query_type, QueryType::Update);
        assert!(query.where_clause.is_some());
    }

    #[test]
    fn test_delete_query_builder() {
        let query = Query::delete()
            .from("users")
            .filter(Expr::eq("id", 1))
            .build();

        assert_eq!(query.query_type, QueryType::Delete);
        assert!(query.where_clause.is_some());
    }
}
