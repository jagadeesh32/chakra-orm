//! SQL generation for Chakra ORM
//!
//! This module provides SQL generation from query objects.

use crate::expr::{AggregateFunc, ArithmeticOp, CompareOp, Expr};
use crate::query::{Order, Query, QueryType};
use crate::types::Value;

/// A SQL fragment with its parameters
#[derive(Debug, Clone)]
pub struct SqlFragment {
    /// The SQL text with placeholders
    pub sql: String,
    /// The parameter values
    pub params: Vec<Value>,
}

impl SqlFragment {
    /// Create a new empty fragment
    pub fn new() -> Self {
        Self {
            sql: String::new(),
            params: Vec::new(),
        }
    }

    /// Create from SQL string
    pub fn from_sql(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            params: Vec::new(),
        }
    }

    /// Append SQL
    pub fn push_sql(&mut self, sql: &str) {
        self.sql.push_str(sql);
    }

    /// Append a parameter
    pub fn push_param(&mut self, value: Value) -> usize {
        self.params.push(value);
        self.params.len()
    }

    /// Combine with another fragment
    pub fn append(&mut self, other: SqlFragment) {
        self.sql.push_str(&other.sql);
        self.params.extend(other.params);
    }
}

impl Default for SqlFragment {
    fn default() -> Self {
        Self::new()
    }
}

/// SQL dialect trait
pub trait Dialect: Send + Sync {
    /// Get the dialect name
    fn name(&self) -> &'static str;

    /// Get the parameter placeholder for the given index
    fn placeholder(&self, index: usize) -> String;

    /// Quote an identifier
    fn quote_identifier(&self, name: &str) -> String;

    /// Generate SQL from a query
    fn generate(&self, query: &Query) -> SqlFragment;

    /// Generate SQL from an expression
    fn generate_expr(&self, expr: &Expr, fragment: &mut SqlFragment);

    /// Check if this dialect supports RETURNING
    fn supports_returning(&self) -> bool;

    /// Check if this dialect supports ILIKE
    fn supports_ilike(&self) -> bool;
}

/// PostgreSQL dialect
#[derive(Debug, Clone, Copy)]
pub struct PostgresDialect;

impl Dialect for PostgresDialect {
    fn name(&self) -> &'static str {
        "postgresql"
    }

    fn placeholder(&self, index: usize) -> String {
        format!("${}", index)
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    fn supports_returning(&self) -> bool {
        true
    }

    fn supports_ilike(&self) -> bool {
        true
    }

    fn generate(&self, query: &Query) -> SqlFragment {
        let mut fragment = SqlFragment::new();

        match query.query_type {
            QueryType::Select => self.generate_select(query, &mut fragment),
            QueryType::Insert => self.generate_insert(query, &mut fragment),
            QueryType::Update => self.generate_update(query, &mut fragment),
            QueryType::Delete => self.generate_delete(query, &mut fragment),
        }

        fragment
    }

    fn generate_expr(&self, expr: &Expr, fragment: &mut SqlFragment) {
        match expr {
            Expr::Column(name) => {
                fragment.push_sql(name);
            }
            Expr::Value(value) => {
                let idx = fragment.push_param(value.clone());
                fragment.push_sql(&self.placeholder(idx));
            }
            Expr::Compare { column, op, value } => {
                fragment.push_sql(column);
                fragment.push_sql(" ");
                fragment.push_sql(op.as_sql());
                if *op != CompareOp::IsNull && *op != CompareOp::IsNotNull {
                    fragment.push_sql(" ");
                    let idx = fragment.push_param(value.clone());
                    fragment.push_sql(&self.placeholder(idx));
                }
            }
            Expr::ColumnCompare { left, op, right } => {
                fragment.push_sql(left);
                fragment.push_sql(" ");
                fragment.push_sql(op.as_sql());
                fragment.push_sql(" ");
                fragment.push_sql(right);
            }
            Expr::Between { column, low, high } => {
                fragment.push_sql(column);
                fragment.push_sql(" BETWEEN ");
                let idx = fragment.push_param(low.clone());
                fragment.push_sql(&self.placeholder(idx));
                fragment.push_sql(" AND ");
                let idx = fragment.push_param(high.clone());
                fragment.push_sql(&self.placeholder(idx));
            }
            Expr::In { column, values, negated } => {
                fragment.push_sql(column);
                if *negated {
                    fragment.push_sql(" NOT IN (");
                } else {
                    fragment.push_sql(" IN (");
                }
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        fragment.push_sql(", ");
                    }
                    let idx = fragment.push_param(value.clone());
                    fragment.push_sql(&self.placeholder(idx));
                }
                fragment.push_sql(")");
            }
            Expr::And(exprs) => {
                fragment.push_sql("(");
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        fragment.push_sql(" AND ");
                    }
                    self.generate_expr(e, fragment);
                }
                fragment.push_sql(")");
            }
            Expr::Or(exprs) => {
                fragment.push_sql("(");
                for (i, e) in exprs.iter().enumerate() {
                    if i > 0 {
                        fragment.push_sql(" OR ");
                    }
                    self.generate_expr(e, fragment);
                }
                fragment.push_sql(")");
            }
            Expr::Not(e) => {
                fragment.push_sql("NOT (");
                self.generate_expr(e, fragment);
                fragment.push_sql(")");
            }
            Expr::Raw(sql) => {
                fragment.push_sql(sql);
            }
            Expr::Function { name, args } => {
                fragment.push_sql(name);
                fragment.push_sql("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        fragment.push_sql(", ");
                    }
                    self.generate_expr(arg, fragment);
                }
                fragment.push_sql(")");
            }
            Expr::Aggregate { function, column, distinct } => {
                fragment.push_sql(function.as_sql());
                fragment.push_sql("(");
                if *distinct {
                    fragment.push_sql("DISTINCT ");
                }
                fragment.push_sql(column);
                fragment.push_sql(")");
            }
            Expr::Arithmetic { left, op, right } => {
                fragment.push_sql("(");
                self.generate_expr(left, fragment);
                fragment.push_sql(" ");
                fragment.push_sql(op.as_sql());
                fragment.push_sql(" ");
                self.generate_expr(right, fragment);
                fragment.push_sql(")");
            }
            Expr::Case { conditions, else_result } => {
                fragment.push_sql("CASE");
                for (when, then) in conditions {
                    fragment.push_sql(" WHEN ");
                    self.generate_expr(when, fragment);
                    fragment.push_sql(" THEN ");
                    self.generate_expr(then, fragment);
                }
                if let Some(else_expr) = else_result {
                    fragment.push_sql(" ELSE ");
                    self.generate_expr(else_expr, fragment);
                }
                fragment.push_sql(" END");
            }
            Expr::Subquery(query) => {
                fragment.push_sql("(");
                let sub = self.generate(query);
                fragment.append(sub);
                fragment.push_sql(")");
            }
        }
    }
}

impl PostgresDialect {
    fn generate_select(&self, query: &Query, fragment: &mut SqlFragment) {
        fragment.push_sql("SELECT ");

        if query.distinct {
            fragment.push_sql("DISTINCT ");
        }

        // Columns
        if query.columns.is_empty() {
            fragment.push_sql("*");
        } else {
            fragment.push_sql(&query.columns.join(", "));
        }

        // FROM
        fragment.push_sql(" FROM ");
        fragment.push_sql(&query.table);
        if let Some(alias) = &query.alias {
            fragment.push_sql(" AS ");
            fragment.push_sql(alias);
        }

        // JOINs
        for join in &query.joins {
            fragment.push_sql(" ");
            fragment.push_sql(join.join_type.as_sql());
            fragment.push_sql(" ");
            fragment.push_sql(&join.table);
            if let Some(alias) = &join.alias {
                fragment.push_sql(" AS ");
                fragment.push_sql(alias);
            }
            fragment.push_sql(" ON ");
            self.generate_expr(&join.on, fragment);
        }

        // WHERE
        if let Some(where_clause) = &query.where_clause {
            fragment.push_sql(" WHERE ");
            self.generate_expr(where_clause, fragment);
        }

        // GROUP BY
        if !query.group_by.is_empty() {
            fragment.push_sql(" GROUP BY ");
            fragment.push_sql(&query.group_by.join(", "));
        }

        // HAVING
        if let Some(having) = &query.having {
            fragment.push_sql(" HAVING ");
            self.generate_expr(having, fragment);
        }

        // ORDER BY
        if !query.order_by.is_empty() {
            fragment.push_sql(" ORDER BY ");
            let order_parts: Vec<String> = query
                .order_by
                .iter()
                .map(|o| format!("{} {}", o.column, o.order.as_sql()))
                .collect();
            fragment.push_sql(&order_parts.join(", "));
        }

        // LIMIT
        if let Some(limit) = query.limit {
            fragment.push_sql(" LIMIT ");
            fragment.push_sql(&limit.to_string());
        }

        // OFFSET
        if let Some(offset) = query.offset {
            fragment.push_sql(" OFFSET ");
            fragment.push_sql(&offset.to_string());
        }

        // FOR UPDATE
        if query.for_update {
            fragment.push_sql(" FOR UPDATE");
        }
    }

    fn generate_insert(&self, query: &Query, fragment: &mut SqlFragment) {
        fragment.push_sql("INSERT INTO ");
        fragment.push_sql(&query.table);

        if let Some(values) = query.values.first() {
            let columns: Vec<&String> = values.keys().collect();
            fragment.push_sql(" (");
            fragment.push_sql(&columns.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", "));
            fragment.push_sql(") VALUES (");

            for (i, col) in columns.iter().enumerate() {
                if i > 0 {
                    fragment.push_sql(", ");
                }
                let value = values.get(*col).unwrap();
                let idx = fragment.push_param(value.clone());
                fragment.push_sql(&self.placeholder(idx));
            }
            fragment.push_sql(")");
        }

        // RETURNING
        if !query.returning.is_empty() {
            fragment.push_sql(" RETURNING ");
            fragment.push_sql(&query.returning.join(", "));
        }
    }

    fn generate_update(&self, query: &Query, fragment: &mut SqlFragment) {
        fragment.push_sql("UPDATE ");
        fragment.push_sql(&query.table);
        fragment.push_sql(" SET ");

        if let Some(values) = query.values.first() {
            let parts: Vec<String> = values
                .iter()
                .map(|(col, val)| {
                    let idx = fragment.push_param(val.clone());
                    format!("{} = {}", col, self.placeholder(idx))
                })
                .collect();
            fragment.push_sql(&parts.join(", "));
        }

        // WHERE
        if let Some(where_clause) = &query.where_clause {
            fragment.push_sql(" WHERE ");
            self.generate_expr(where_clause, fragment);
        }

        // RETURNING
        if !query.returning.is_empty() {
            fragment.push_sql(" RETURNING ");
            fragment.push_sql(&query.returning.join(", "));
        }
    }

    fn generate_delete(&self, query: &Query, fragment: &mut SqlFragment) {
        fragment.push_sql("DELETE FROM ");
        fragment.push_sql(&query.table);

        // WHERE
        if let Some(where_clause) = &query.where_clause {
            fragment.push_sql(" WHERE ");
            self.generate_expr(where_clause, fragment);
        }

        // RETURNING
        if !query.returning.is_empty() {
            fragment.push_sql(" RETURNING ");
            fragment.push_sql(&query.returning.join(", "));
        }
    }
}

/// MySQL dialect
#[derive(Debug, Clone, Copy)]
pub struct MySqlDialect;

impl Dialect for MySqlDialect {
    fn name(&self) -> &'static str {
        "mysql"
    }

    fn placeholder(&self, _index: usize) -> String {
        "?".to_string()
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("`{}`", name.replace('`', "``"))
    }

    fn supports_returning(&self) -> bool {
        false
    }

    fn supports_ilike(&self) -> bool {
        false
    }

    fn generate(&self, query: &Query) -> SqlFragment {
        // Similar to PostgreSQL but with MySQL-specific syntax
        // For now, use a simplified implementation
        let pg = PostgresDialect;
        let mut fragment = pg.generate(query);

        // Replace $N with ?
        let mut new_sql = String::new();
        let mut in_placeholder = false;
        for c in fragment.sql.chars() {
            if c == '$' {
                in_placeholder = true;
                new_sql.push('?');
            } else if in_placeholder && c.is_ascii_digit() {
                // Skip the number
            } else {
                in_placeholder = false;
                new_sql.push(c);
            }
        }
        fragment.sql = new_sql;

        // Replace ILIKE with LIKE (case-insensitive by default in MySQL)
        fragment.sql = fragment.sql.replace(" ILIKE ", " LIKE ");

        fragment
    }

    fn generate_expr(&self, expr: &Expr, fragment: &mut SqlFragment) {
        PostgresDialect.generate_expr(expr, fragment);
    }
}

/// SQLite dialect
#[derive(Debug, Clone, Copy)]
pub struct SqliteDialect;

impl Dialect for SqliteDialect {
    fn name(&self) -> &'static str {
        "sqlite"
    }

    fn placeholder(&self, index: usize) -> String {
        format!("?{}", index)
    }

    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    fn supports_returning(&self) -> bool {
        true // SQLite 3.35+
    }

    fn supports_ilike(&self) -> bool {
        false // Use LIKE with COLLATE NOCASE
    }

    fn generate(&self, query: &Query) -> SqlFragment {
        PostgresDialect.generate(query)
    }

    fn generate_expr(&self, expr: &Expr, fragment: &mut SqlFragment) {
        PostgresDialect.generate_expr(expr, fragment);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Query;

    #[test]
    fn test_select_query() {
        let query = Query::select()
            .from("users")
            .columns(&["id", "name"])
            .filter(Expr::eq("is_active", true))
            .order_by_desc("created_at")
            .limit(10)
            .build();

        let dialect = PostgresDialect;
        let fragment = dialect.generate(&query);

        assert!(fragment.sql.contains("SELECT id, name FROM users"));
        assert!(fragment.sql.contains("WHERE is_active = $1"));
        assert!(fragment.sql.contains("ORDER BY created_at DESC"));
        assert!(fragment.sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_insert_query() {
        let query = Query::insert()
            .table("users")
            .set("name", "Alice")
            .set("email", "alice@example.com")
            .returning(&["id"])
            .build();

        let dialect = PostgresDialect;
        let fragment = dialect.generate(&query);

        assert!(fragment.sql.contains("INSERT INTO users"));
        assert!(fragment.sql.contains("RETURNING id"));
    }

    #[test]
    fn test_and_expression() {
        let expr = Expr::eq("a", 1).and(Expr::eq("b", 2));
        let dialect = PostgresDialect;
        let mut fragment = SqlFragment::new();
        dialect.generate_expr(&expr, &mut fragment);

        assert!(fragment.sql.contains("AND"));
        assert_eq!(fragment.params.len(), 2);
    }
}
