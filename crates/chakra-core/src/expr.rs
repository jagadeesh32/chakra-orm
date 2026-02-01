//! Expression system for Chakra ORM
//!
//! This module provides:
//! - `Expr` - Expression tree for WHERE clauses
//! - `F` - Field reference expressions
//! - `Q` - Query expressions for complex conditions

use crate::types::Value;
use serde::{Deserialize, Serialize};

/// Comparison operators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Like,
    ILike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
}

impl CompareOp {
    pub fn as_sql(&self) -> &'static str {
        match self {
            CompareOp::Eq => "=",
            CompareOp::Ne => "!=",
            CompareOp::Lt => "<",
            CompareOp::Lte => "<=",
            CompareOp::Gt => ">",
            CompareOp::Gte => ">=",
            CompareOp::Like => "LIKE",
            CompareOp::ILike => "ILIKE",
            CompareOp::In => "IN",
            CompareOp::NotIn => "NOT IN",
            CompareOp::IsNull => "IS NULL",
            CompareOp::IsNotNull => "IS NOT NULL",
            CompareOp::Between => "BETWEEN",
        }
    }
}

/// Expression tree for SQL conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Column reference
    Column(String),

    /// Literal value
    Value(Value),

    /// Comparison: column op value
    Compare {
        column: String,
        op: CompareOp,
        value: Value,
    },

    /// Column comparison: column1 op column2
    ColumnCompare {
        left: String,
        op: CompareOp,
        right: String,
    },

    /// BETWEEN: column BETWEEN low AND high
    Between {
        column: String,
        low: Value,
        high: Value,
    },

    /// IN: column IN (values)
    In {
        column: String,
        values: Vec<Value>,
        negated: bool,
    },

    /// AND of multiple expressions
    And(Vec<Expr>),

    /// OR of multiple expressions
    Or(Vec<Expr>),

    /// NOT expression
    Not(Box<Expr>),

    /// Raw SQL expression
    Raw(String),

    /// Function call
    Function {
        name: String,
        args: Vec<Expr>,
    },

    /// Aggregate function
    Aggregate {
        function: AggregateFunc,
        column: String,
        distinct: bool,
    },

    /// Arithmetic operation
    Arithmetic {
        left: Box<Expr>,
        op: ArithmeticOp,
        right: Box<Expr>,
    },

    /// Case expression
    Case {
        conditions: Vec<(Expr, Expr)>,
        else_result: Option<Box<Expr>>,
    },

    /// Subquery
    Subquery(Box<crate::query::Query>),
}

/// Aggregate functions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunc {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    ArrayAgg,
    StringAgg,
}

impl AggregateFunc {
    pub fn as_sql(&self) -> &'static str {
        match self {
            AggregateFunc::Count => "COUNT",
            AggregateFunc::Sum => "SUM",
            AggregateFunc::Avg => "AVG",
            AggregateFunc::Min => "MIN",
            AggregateFunc::Max => "MAX",
            AggregateFunc::ArrayAgg => "ARRAY_AGG",
            AggregateFunc::StringAgg => "STRING_AGG",
        }
    }
}

/// Arithmetic operators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl ArithmeticOp {
    pub fn as_sql(&self) -> &'static str {
        match self {
            ArithmeticOp::Add => "+",
            ArithmeticOp::Sub => "-",
            ArithmeticOp::Mul => "*",
            ArithmeticOp::Div => "/",
            ArithmeticOp::Mod => "%",
        }
    }
}

impl Expr {
    // Comparison constructors

    /// Create an equality expression
    pub fn eq(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::Eq,
            value: value.into(),
        }
    }

    /// Create a not-equal expression
    pub fn ne(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::Ne,
            value: value.into(),
        }
    }

    /// Create a less-than expression
    pub fn lt(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::Lt,
            value: value.into(),
        }
    }

    /// Create a less-than-or-equal expression
    pub fn lte(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::Lte,
            value: value.into(),
        }
    }

    /// Create a greater-than expression
    pub fn gt(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::Gt,
            value: value.into(),
        }
    }

    /// Create a greater-than-or-equal expression
    pub fn gte(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::Gte,
            value: value.into(),
        }
    }

    /// Create a LIKE expression
    pub fn like(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::Like,
            value: Value::String(pattern.into()),
        }
    }

    /// Create an ILIKE expression (case-insensitive LIKE)
    pub fn ilike(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::ILike,
            value: Value::String(pattern.into()),
        }
    }

    /// Create an IS NULL expression
    pub fn is_null(column: impl Into<String>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::IsNull,
            value: Value::Null,
        }
    }

    /// Create an IS NOT NULL expression
    pub fn is_not_null(column: impl Into<String>) -> Self {
        Expr::Compare {
            column: column.into(),
            op: CompareOp::IsNotNull,
            value: Value::Null,
        }
    }

    /// Create an IN expression
    pub fn is_in<V: Into<Value>>(column: impl Into<String>, values: Vec<V>) -> Self {
        Expr::In {
            column: column.into(),
            values: values.into_iter().map(Into::into).collect(),
            negated: false,
        }
    }

    /// Create a NOT IN expression
    pub fn not_in<V: Into<Value>>(column: impl Into<String>, values: Vec<V>) -> Self {
        Expr::In {
            column: column.into(),
            values: values.into_iter().map(Into::into).collect(),
            negated: true,
        }
    }

    /// Create a BETWEEN expression
    pub fn between(
        column: impl Into<String>,
        low: impl Into<Value>,
        high: impl Into<Value>,
    ) -> Self {
        Expr::Between {
            column: column.into(),
            low: low.into(),
            high: high.into(),
        }
    }

    // Logical operators

    /// Combine with AND
    pub fn and(self, other: Expr) -> Self {
        match self {
            Expr::And(mut exprs) => {
                exprs.push(other);
                Expr::And(exprs)
            }
            _ => Expr::And(vec![self, other]),
        }
    }

    /// Combine with OR
    pub fn or(self, other: Expr) -> Self {
        match self {
            Expr::Or(mut exprs) => {
                exprs.push(other);
                Expr::Or(exprs)
            }
            _ => Expr::Or(vec![self, other]),
        }
    }

    /// Negate the expression
    pub fn not(self) -> Self {
        Expr::Not(Box::new(self))
    }

    /// Create a raw SQL expression
    pub fn raw(sql: impl Into<String>) -> Self {
        Expr::Raw(sql.into())
    }

    /// Create a column reference
    pub fn column(name: impl Into<String>) -> Self {
        Expr::Column(name.into())
    }

    /// Create a literal value
    pub fn value(val: impl Into<Value>) -> Self {
        Expr::Value(val.into())
    }
}

/// Field reference (F object) for column references in expressions
#[derive(Debug, Clone)]
pub struct F {
    column: String,
}

impl F {
    /// Create a new field reference
    pub fn new(column: impl Into<String>) -> Self {
        Self {
            column: column.into(),
        }
    }

    /// Shorthand for creating a field reference
    pub fn col(column: impl Into<String>) -> Self {
        Self::new(column)
    }

    /// Get the column name
    pub fn column(&self) -> &str {
        &self.column
    }

    /// Convert to an expression
    pub fn to_expr(&self) -> Expr {
        Expr::Column(self.column.clone())
    }

    // Comparison methods

    pub fn eq(&self, value: impl Into<Value>) -> Expr {
        Expr::eq(&self.column, value)
    }

    pub fn ne(&self, value: impl Into<Value>) -> Expr {
        Expr::ne(&self.column, value)
    }

    pub fn lt(&self, value: impl Into<Value>) -> Expr {
        Expr::lt(&self.column, value)
    }

    pub fn lte(&self, value: impl Into<Value>) -> Expr {
        Expr::lte(&self.column, value)
    }

    pub fn gt(&self, value: impl Into<Value>) -> Expr {
        Expr::gt(&self.column, value)
    }

    pub fn gte(&self, value: impl Into<Value>) -> Expr {
        Expr::gte(&self.column, value)
    }

    pub fn is_in<V: Into<Value>>(&self, values: Vec<V>) -> Expr {
        Expr::is_in(&self.column, values)
    }

    pub fn between(&self, low: impl Into<Value>, high: impl Into<Value>) -> Expr {
        Expr::between(&self.column, low, high)
    }

    pub fn is_null(&self) -> Expr {
        Expr::is_null(&self.column)
    }

    pub fn is_not_null(&self) -> Expr {
        Expr::is_not_null(&self.column)
    }

    pub fn like(&self, pattern: impl Into<String>) -> Expr {
        Expr::like(&self.column, pattern)
    }

    pub fn starts_with(&self, prefix: impl AsRef<str>) -> Expr {
        Expr::like(&self.column, format!("{}%", prefix.as_ref()))
    }

    pub fn ends_with(&self, suffix: impl AsRef<str>) -> Expr {
        Expr::like(&self.column, format!("%{}", suffix.as_ref()))
    }

    pub fn contains(&self, substring: impl AsRef<str>) -> Expr {
        Expr::like(&self.column, format!("%{}%", substring.as_ref()))
    }

    // Arithmetic

    pub fn add(&self, value: impl Into<Value>) -> Expr {
        Expr::Arithmetic {
            left: Box::new(Expr::Column(self.column.clone())),
            op: ArithmeticOp::Add,
            right: Box::new(Expr::Value(value.into())),
        }
    }

    pub fn sub(&self, value: impl Into<Value>) -> Expr {
        Expr::Arithmetic {
            left: Box::new(Expr::Column(self.column.clone())),
            op: ArithmeticOp::Sub,
            right: Box::new(Expr::Value(value.into())),
        }
    }
}

/// Query object (Q) for complex boolean expressions
#[derive(Debug, Clone)]
pub struct Q {
    expr: Expr,
}

impl Q {
    /// Create a new Q object with an equality condition
    pub fn new(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            expr: Expr::eq(column, value),
        }
    }

    /// Create from an expression
    pub fn from_expr(expr: Expr) -> Self {
        Self { expr }
    }

    /// Get the inner expression
    pub fn into_expr(self) -> Expr {
        self.expr
    }

    /// Combine with AND
    pub fn and(self, other: Q) -> Self {
        Self {
            expr: self.expr.and(other.expr),
        }
    }

    /// Combine with OR
    pub fn or(self, other: Q) -> Self {
        Self {
            expr: self.expr.or(other.expr),
        }
    }

    /// Negate
    pub fn not(self) -> Self {
        Self {
            expr: self.expr.not(),
        }
    }
}

// Implement bitwise operators for Q
impl std::ops::BitAnd for Q {
    type Output = Q;

    fn bitand(self, rhs: Q) -> Q {
        self.and(rhs)
    }
}

impl std::ops::BitOr for Q {
    type Output = Q;

    fn bitor(self, rhs: Q) -> Q {
        self.or(rhs)
    }
}

impl std::ops::Not for Q {
    type Output = Q;

    fn not(self) -> Q {
        Q::from_expr(self.expr.not())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_eq() {
        let expr = Expr::eq("name", "Alice");
        match expr {
            Expr::Compare { column, op, value } => {
                assert_eq!(column, "name");
                assert_eq!(op, CompareOp::Eq);
                assert_eq!(value.as_str(), Some("Alice"));
            }
            _ => panic!("Expected Compare"),
        }
    }

    #[test]
    fn test_expr_and() {
        let expr = Expr::eq("a", 1).and(Expr::eq("b", 2));
        match expr {
            Expr::And(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_f_object() {
        let f = F::col("age");
        let expr = f.gte(18);
        match expr {
            Expr::Compare { column, op, .. } => {
                assert_eq!(column, "age");
                assert_eq!(op, CompareOp::Gte);
            }
            _ => panic!("Expected Compare"),
        }
    }

    #[test]
    fn test_q_object() {
        let q1 = Q::new("a", 1);
        let q2 = Q::new("b", 2);
        let combined = q1 & q2;

        match combined.into_expr() {
            Expr::And(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_q_or() {
        let q1 = Q::new("a", 1);
        let q2 = Q::new("b", 2);
        let combined = q1 | q2;

        match combined.into_expr() {
            Expr::Or(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected Or"),
        }
    }
}
