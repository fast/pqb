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
use crate::types::write_iden;
use crate::types::write_table_name;
use crate::value::Value;
use crate::value::write_value;
use crate::writer::SqlWriter;

/// An arbitrary, dynamically-typed SQL expression.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Expr {
    /// A reference to a column.
    Column(ColumnRef),
    /// A literal value.
    Value(Value),
}

/// Create a new column expression from a value.
pub fn value<T>(value: T) -> Expr
where
    T: Into<Value>,
{
    Expr::Value(value.into())
}

pub(crate) fn write_expr<W: SqlWriter>(w: &mut W, expr: &Expr) {
    match expr {
        Expr::Column(col) => {
            write_column_ref(w, col);
        }
        Expr::Value(value) => {
            write_value(w, value.clone());
        }
    }
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
