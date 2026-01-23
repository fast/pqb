// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Building blocks of SQL statements.
//!
//! [`Expr`] is an arbitrary, dynamically-typed SQL expression.
//! It can be used in select fields, where clauses and many other places.

use crate::types::ColumnName;
use crate::types::ColumnRef;
use crate::types::IntoColumnRef;
use crate::types::write_iden;
use crate::types::write_table_name;
use crate::value::Value;
use crate::value::write_value;
use crate::writer::SqlWriter;

/// An arbitrary, dynamically-typed SQL expression.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)] // trivial
pub enum Expr {
    Column(ColumnRef),
    Value(Value),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

/// # Expression constructors
impl Expr {
    /// Express a [`Value`], returning a [`Expr`].
    pub fn value<T>(value: T) -> Expr
    where
        T: Into<Value>,
    {
        Expr::Value(value.into())
    }

    /// Express the target column, returning a [`Expr`].
    pub fn column<T>(n: T) -> Self
    where
        T: IntoColumnRef,
    {
        Self::Column(n.into_column_ref())
    }
}

/// # Expression combinators
impl Expr {
    /// Create any binary operation.
    pub fn binary<R>(self, op: BinaryOp, rhs: R) -> Self
    where
        R: Into<Expr>,
    {
        Expr::Binary(Box::new(self), op, Box::new(rhs.into()))
    }

    /// Express a logical `AND` operation.
    pub fn and<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::And, right)
    }

    /// Express a logical `OR` operation.
    pub fn or<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Or, right)
    }

    /// Express an equal (`=`) expression.
    pub fn eq<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Equal, right)
    }

    /// Express a not equal (`<>`) expression.
    pub fn ne<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::NotEqual, right)
    }
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)] // trivial
pub enum BinaryOp {
    And,
    Or,
    Equal,
    NotEqual,
}

impl Expr {
    pub(crate) fn from_conditions(conditions: Vec<Expr>) -> Option<Expr> {
        conditions
            .into_iter()
            .reduce(|lhs, rhs| lhs.binary(BinaryOp::And, rhs))
    }
}

impl<T> From<T> for Expr
where
    T: Into<Value>,
{
    fn from(v: T) -> Self {
        Expr::Value(v.into())
    }
}

pub(crate) fn write_expr<W: SqlWriter>(w: &mut W, expr: &Expr) {
    match expr {
        Expr::Column(col) => {
            write_column_ref(w, col);
        }
        Expr::Value(value) => {
            write_value(w, value.clone());
        }
        Expr::Binary(lhs, op, rhs) => write_binary_expr(w, lhs, op, rhs),
    }
}

fn write_binary_expr<W: SqlWriter>(w: &mut W, lhs: &Expr, op: &BinaryOp, rhs: &Expr) {
    let left_paren = !well_known_no_parentheses(lhs);
    if left_paren {
        w.push_char('(');
    }
    write_expr(w, lhs);
    if left_paren {
        w.push_char(')');
    }

    w.push_char(' ');
    write_binary_op(w, op);
    w.push_char(' ');

    let right_paren = !well_known_no_parentheses(rhs);
    if right_paren {
        w.push_char('(');
    }
    write_expr(w, rhs);
    if right_paren {
        w.push_char(')');
    }
}

fn write_binary_op<W: SqlWriter>(w: &mut W, op: &BinaryOp) {
    w.push_str(match op {
        BinaryOp::And => "AND",
        BinaryOp::Or => "OR",
        BinaryOp::Equal => "=",
        BinaryOp::NotEqual => "<>",
    })
}

fn write_column_ref<W: SqlWriter>(w: &mut W, col: &ColumnRef) {
    match col {
        ColumnRef::Column(ColumnName(table_name, column)) => {
            if let Some(table_name) = table_name {
                write_table_name(w, table_name);
                w.push_char('.');
            }
            write_iden(w, column);
        }
        ColumnRef::Asterisk(table_name) => {
            if let Some(table_name) = table_name {
                write_table_name(w, table_name);
                w.push_char('.');
            }
            w.push_char('*');
        }
    }
}

fn well_known_no_parentheses(expr: &Expr) -> bool {
    matches!(expr, Expr::Column(_) | Expr::Value(_))
}
