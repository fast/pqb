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
use crate::query::Select;
use crate::query::write_returning;
use crate::query::write_select;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::IntoTableRef;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// Insert any new rows into an existing table.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Insert {
    table: Option<TableRef>,
    columns: Vec<Iden>,
    source: Option<InsertValueSource>,
    defaults: Option<u32>,
    returning: Option<Returning>,
}

impl Insert {
    /// Create a new INSERT statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_insert(&mut w, self);
        w
    }

    /// Convert the insert statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_insert(&mut sql, self);
        sql
    }

    /// Specify which table to insert into.
    pub fn into_table<T>(mut self, table: T) -> Self
    where
        T: IntoTableRef,
    {
        self.table = Some(table.into());
        self
    }

    /// Specify what columns to insert.
    pub fn columns<T, I>(mut self, cols: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoIden,
    {
        for col in cols {
            self.columns.push(col.into_iden());
        }
        self
    }

    /// RETURNING expressions.
    pub fn returning(mut self, returning: Returning) -> Self {
        self.returning = Some(returning);
        self
    }

    /// Specify a row of values to be inserted.
    ///
    /// # Panics
    ///
    /// Panics if the number of values does not match the number of columns specified.
    pub fn values<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Expr>,
    {
        let values = values.into_iter().collect::<Vec<_>>();
        assert_eq!(values.len(), self.columns.len());
        if !values.is_empty() {
            if let Some(InsertValueSource::Values(vs)) = &mut self.source {
                vs.push(values);
            } else {
                self.source = Some(InsertValueSource::Values(vec![values]));
            }
        }
        self
    }

    /// Specify a SELECT statement to insert rows from.
    ///
    /// # Panics
    ///
    /// Panics if the number of selected columns does not match the number of columns specified.
    pub fn select_from<S>(mut self, select: S) -> Self
    where
        S: Into<Select>,
    {
        let select = select.into();
        assert_eq!(select.columns_len(), self.columns.len());
        self.source = Some(InsertValueSource::Select(Box::new(select)));
        self
    }

    /// Insert `n` rows with default values if columns and values are not supplied.
    pub fn or_default_values(mut self, n: u32) -> Self {
        self.defaults = Some(n);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InsertValueSource {
    Values(Vec<Vec<Expr>>),
    Select(Box<Select>),
}

fn write_insert<W: SqlWriter>(w: &mut W, insert: &Insert) {
    w.push_str("INSERT ");

    if let Some(table) = &insert.table {
        w.push_str("INTO ");
        write_table_ref(w, table);
    }

    if insert.defaults.unwrap_or_default() != 0
        && insert.columns.is_empty()
        && insert.source.is_none()
    {
        let num_rows = insert.defaults.unwrap_or_default();
        w.push_str(" VALUES ");
        for i in 0..num_rows {
            if i > 0 {
                w.push_str(", ");
            }
            w.push_str("(DEFAULT)");
        }
    } else {
        w.push_str(" (");
        for (i, col) in insert.columns.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            write_iden(w, col);
        }
        w.push_str(")");

        if let Some(source) = &insert.source {
            w.push_char(' ');
            match source {
                InsertValueSource::Values(rows) => {
                    w.push_str("VALUES ");
                    for (i, row) in rows.iter().enumerate() {
                        if i > 0 {
                            w.push_str(", ");
                        }
                        w.push_char('(');
                        for (j, expr) in row.iter().enumerate() {
                            if j > 0 {
                                w.push_str(", ");
                            }
                            write_expr(w, expr);
                        }
                        w.push_char(')');
                    }
                }
                InsertValueSource::Select(select) => write_select(w, select),
            }
        }

        if let Some(returning) = &insert.returning {
            write_returning(w, returning);
        }
    }
}
