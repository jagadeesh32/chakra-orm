//! Query builder for Python bindings

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Python query builder
#[pyclass]
#[derive(Clone)]
pub struct PyQueryBuilder {
    table: String,
    columns: Vec<String>,
    filters: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[pymethods]
impl PyQueryBuilder {
    /// Create a new query builder
    #[new]
    fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: Vec::new(),
            filters: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Select specific columns
    fn select(&mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self.clone()
    }

    /// Add a filter condition
    fn filter(&mut self, condition: &str) -> Self {
        self.filters.push(condition.to_string());
        self.clone()
    }

    /// Add WHERE clause
    fn where_(&mut self, column: &str, value: &str) -> Self {
        self.filters.push(format!("{} = {}", column, value));
        self.clone()
    }

    /// Add ORDER BY
    fn order_by(&mut self, column: &str, desc: bool) -> Self {
        let order = if desc {
            format!("{} DESC", column)
        } else {
            format!("{} ASC", column)
        };
        self.order_by.push(order);
        self.clone()
    }

    /// Set LIMIT
    fn limit(&mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self.clone()
    }

    /// Set OFFSET
    fn offset(&mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self.clone()
    }

    /// Build the SQL query
    fn build(&self) -> String {
        let mut sql = String::from("SELECT ");

        if self.columns.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.columns.join(", "));
        }

        sql.push_str(" FROM ");
        sql.push_str(&self.table);

        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.filters.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    /// Get string representation
    fn __str__(&self) -> String {
        self.build()
    }

    /// Get representation
    fn __repr__(&self) -> String {
        format!("QueryBuilder('{}')", self.build())
    }
}

/// Field reference for expressions (like Django's F object)
#[pyclass]
#[derive(Clone)]
pub struct F {
    column: String,
}

#[pymethods]
impl F {
    #[new]
    fn new(column: &str) -> Self {
        Self {
            column: column.to_string(),
        }
    }

    fn eq(&self, value: &str) -> String {
        format!("{} = {}", self.column, value)
    }

    fn ne(&self, value: &str) -> String {
        format!("{} != {}", self.column, value)
    }

    fn lt(&self, value: &str) -> String {
        format!("{} < {}", self.column, value)
    }

    fn lte(&self, value: &str) -> String {
        format!("{} <= {}", self.column, value)
    }

    fn gt(&self, value: &str) -> String {
        format!("{} > {}", self.column, value)
    }

    fn gte(&self, value: &str) -> String {
        format!("{} >= {}", self.column, value)
    }

    fn is_null(&self) -> String {
        format!("{} IS NULL", self.column)
    }

    fn is_not_null(&self) -> String {
        format!("{} IS NOT NULL", self.column)
    }

    fn __str__(&self) -> String {
        self.column.clone()
    }

    fn __repr__(&self) -> String {
        format!("F('{}')", self.column)
    }
}

/// Query expression (like Django's Q object)
#[pyclass]
#[derive(Clone)]
pub struct Q {
    expression: String,
}

#[pymethods]
impl Q {
    #[new]
    fn new(expression: &str) -> Self {
        Self {
            expression: expression.to_string(),
        }
    }

    fn and_(&self, other: &Q) -> Q {
        Q {
            expression: format!("({} AND {})", self.expression, other.expression),
        }
    }

    fn or_(&self, other: &Q) -> Q {
        Q {
            expression: format!("({} OR {})", self.expression, other.expression),
        }
    }

    fn not_(&self) -> Q {
        Q {
            expression: format!("NOT ({})", self.expression),
        }
    }

    fn __str__(&self) -> String {
        self.expression.clone()
    }

    fn __repr__(&self) -> String {
        format!("Q('{}')", self.expression)
    }
}
