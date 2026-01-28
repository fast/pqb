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

use crate::SqlWriterValues;
use crate::expr::Expr;
use crate::expr::write_expr;
use crate::query::Returning;
use crate::query::With;
use crate::query::write_returning;
use crate::query::write_with;
use crate::types::IntoTableRef;
use crate::types::TableRef;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// Delete existing rows from the table.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Delete {
    table: Option<TableRef>,
    conditions: Vec<Expr>,
    returning: Option<Returning>,
    with: Option<With>,
}

impl Delete {
    /// Create a new DELETE query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_delete(&mut w, self);
        w
    }

    /// Convert the delete statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_delete(&mut sql, self);
        sql
    }

    /// Specify which table to delete from.
    pub fn from_table<T>(mut self, table: T) -> Self
    where
        T: IntoTableRef,
    {
        self.table = Some(table.into());
        self
    }

    /// And where condition.
    pub fn and_where<T>(mut self, expr: T) -> Self
    where
        T: Into<Expr>,
    {
        self.conditions.push(expr.into());
        self
    }

    /// RETURNING expressions.
    pub fn returning(mut self, returning_cols: Returning) -> Self {
        self.returning = Some(returning_cols);
        self
    }

    /// WITH clause.
    pub fn with(mut self, with: With) -> Self {
        self.with = Some(with);
        self
    }
}

fn write_delete<W: SqlWriter>(w: &mut W, delete: &Delete) {
    if let Some(with) = &delete.with {
        write_with(w, with);
        w.push_char(' ');
    }

    w.push_str("DELETE ");

    if let Some(table) = &delete.table {
        w.push_str("FROM ");
        write_table_ref(w, table);
    }

    if let Some(condition) = Expr::from_conditions(delete.conditions.clone()) {
        w.push_str(" WHERE ");
        write_expr(w, &condition);
    }

    if let Some(returning) = &delete.returning {
        write_returning(w, returning);
    }
}
