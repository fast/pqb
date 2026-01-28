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
use crate::query::write_returning;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::IntoTableRef;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// Update existing rows in the table.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Update {
    table: Option<TableRef>,
    values: Vec<(Iden, Expr)>,
    conditions: Vec<Expr>,
    returning: Option<Returning>,
}

impl Update {
    /// Create a new UPDATE query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_update(&mut w, self);
        w
    }

    /// Convert the update statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_update(&mut sql, self);
        sql
    }

    /// Specify which table to update.
    pub fn table<T>(mut self, table: T) -> Self
    where
        T: IntoTableRef,
    {
        self.table = Some(table.into());
        self
    }

    /// Update column values.
    pub fn values<T, I>(mut self, values: I) -> Self
    where
        T: IntoIden,
        I: IntoIterator<Item = (T, Expr)>,
    {
        for (k, v) in values.into_iter() {
            self.values.push((k.into_iden(), v));
        }
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
    pub fn returning(mut self, returning: Returning) -> Self {
        self.returning = Some(returning);
        self
    }
}

fn write_update<W: SqlWriter>(w: &mut W, update: &Update) {
    w.push_str("UPDATE ");

    if let Some(table) = &update.table {
        write_table_ref(w, table);
    }

    if !update.values.is_empty() {
        w.push_str(" SET ");
        for (i, (col, val)) in update.values.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            write_iden(w, col);
            w.push_str(" = ");
            write_expr(w, val);
        }
    }

    if let Some(condition) = Expr::from_conditions(update.conditions.clone()) {
        w.push_str(" WHERE ");
        write_expr(w, &condition);
    }

    if let Some(returning) = &update.returning {
        w.push_str(" RETURNING ");
        write_returning(w, returning);
    }
}
