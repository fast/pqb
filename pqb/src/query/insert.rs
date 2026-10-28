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
use crate::query::OnConflict;
use crate::query::Returning;
use crate::query::Select;
use crate::query::With;
use crate::query::write_on_conflict;
use crate::query::write_returning;
use crate::query::write_select;
use crate::query::write_with;
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
    on_conflict: Option<OnConflict>,
    defaults: bool,
    pub(super) returning: Option<Returning>,
    with: Option<With>,
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

    /// ON CONFLICT expression
    pub fn on_conflict(mut self, on_conflict: OnConflict) -> Self {
        self.on_conflict = Some(on_conflict);
        self
    }

    /// RETURNING expressions.
    pub fn returning(mut self, returning: Returning) -> Self {
        self.returning = Some(returning);
        self
    }

    /// WITH clause.
    pub fn with(mut self, with: With) -> Self {
        self.with = Some(with);
        self
    }

    /// Specify a row of values to be inserted.
    ///
    /// # Panics
    ///
    /// Panics if the number of values does not match the number of columns specified.
    pub fn values<I>(self, values: I) -> Self
    where
        I: IntoIterator<Item = Expr>,
    {
        self.try_values(values).expect("invalid INSERT row")
    }

    /// Append a row, returning a descriptive error for empty or inconsistent rows.
    ///
    /// If columns are omitted, their order is the table's declared order. The first row sets the
    /// width.
    pub fn try_values<I>(mut self, values: I) -> Result<Self, InsertError>
    where
        I: IntoIterator<Item = Expr>,
    {
        let values: Vec<_> = values.into_iter().collect();
        if values.is_empty() {
            return Err(InsertError::EmptyRow);
        }
        let width = if !self.columns.is_empty() {
            Some(self.columns.len())
        } else if let Some(InsertValueSource::Values(rows)) = &self.source {
            Some(rows[0].len())
        } else {
            None
        };
        if let Some(expected) = width
            && values.len() != expected
        {
            return Err(InsertError::ColumnCount {
                expected,
                actual: values.len(),
            });
        }
        if let Some(InsertValueSource::Values(rows)) = &mut self.source {
            rows.push(values);
        } else {
            self.source = Some(InsertValueSource::Values(vec![values]));
        }
        Ok(self)
    }

    /// Append several rows. Panics on an empty or inconsistent row, like [`Self::values`].
    pub fn rows<I, R>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = Expr>,
    {
        for row in rows {
            self = self.values(row);
        }
        self
    }

    /// Specify a SELECT statement to insert rows from.
    ///
    /// # Panics
    ///
    /// Rendering panics if known projection and target widths differ. Wildcard expansion is checked
    /// by PostgreSQL.
    pub fn select_from<S>(mut self, select: S) -> Self
    where
        S: Into<Select>,
    {
        self.source = Some(InsertValueSource::Select(Box::new(select.into())));
        self
    }

    /// Use DEFAULT VALUES as the source, replacing any previous source.
    pub fn default_values(mut self) -> Self {
        self.columns.clear();
        self.source = None;
        self.defaults = true;
        self
    }

    /// Insert one default row if columns and values are not supplied; zero disables the fallback.
    ///
    /// # Panics
    ///
    /// Panics for `n > 1`. PostgreSQL's DEFAULT VALUES inserts exactly one row.
    pub fn or_default_values(mut self, n: u32) -> Self {
        assert!(n <= 1, "DEFAULT VALUES inserts exactly one row");
        self.defaults = n == 1;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InsertValueSource {
    Values(Vec<Vec<Expr>>),
    Select(Box<Select>),
}

/// An invalid row supplied to [`Insert::try_values`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertError {
    /// A VALUES row cannot be empty. Use DEFAULT VALUES to insert a default row.
    EmptyRow,
    /// The row width differs from the target columns or preceding rows.
    ColumnCount {
        /// Required number of expressions.
        expected: usize,
        /// Supplied number of expressions.
        actual: usize,
    },
}

impl std::fmt::Display for InsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyRow => f.write_str("INSERT row is empty; use default_values()"),
            Self::ColumnCount { expected, actual } => write!(
                f,
                "INSERT row has {actual} expressions; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for InsertError {}

pub(crate) fn write_insert<W: SqlWriter>(w: &mut W, insert: &Insert) {
    assert!(
        matches!(&insert.table, Some(TableRef::Table(..))),
        "INSERT requires a target table"
    );
    assert!(
        insert.source.is_some() || (insert.defaults && insert.columns.is_empty()),
        "INSERT requires VALUES, SELECT or DEFAULT VALUES"
    );
    if let Some(InsertValueSource::Values(rows)) = &insert.source
        && !insert.columns.is_empty()
    {
        assert!(
            rows.iter().all(|row| row.len() == insert.columns.len()),
            "INSERT row width does not match target columns"
        );
    }
    if let Some(InsertValueSource::Select(select)) = &insert.source
        && !insert.columns.is_empty()
        && let Some(width) = select.columns_len()
    {
        assert_eq!(
            width,
            insert.columns.len(),
            "INSERT projection width does not match target columns"
        );
    }
    if let Some(with) = &insert.with {
        write_with(w, with);
        w.push_char(' ');
    }

    w.push_str("INSERT ");

    if let Some(table) = &insert.table {
        w.push_str("INTO ");
        write_table_ref(w, table);
    }

    if insert.defaults && insert.columns.is_empty() && insert.source.is_none() {
        w.push_str(" DEFAULT VALUES");
    } else {
        if !insert.columns.is_empty() {
            w.push_str(" (");
            for (i, col) in insert.columns.iter().enumerate() {
                if i > 0 {
                    w.push_str(", ");
                }
                write_iden(w, col);
            }
            w.push_str(")");
        }

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
    }

    if let Some(on_conflict) = &insert.on_conflict {
        write_on_conflict(w, on_conflict);
    }

    if let Some(returning) = &insert.returning {
        write_returning(w, returning);
    }
}
