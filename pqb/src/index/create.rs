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
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// CREATE INDEX statement builder.
#[derive(Default, Debug, Clone)]
pub struct CreateIndex {
    table: Option<TableRef>,
    columns: Vec<Iden>,
    primary: bool,
    unique: bool,
}

impl CreateIndex {
    /// Create a new CREATE INDEX statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_create_index(&mut w, self);
        w
    }

    /// Convert the CREATE INDEX statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_create_index(&mut sql, self);
        sql
    }

    /// Specify the table to create the index on.
    pub fn table<T>(mut self, table: T) -> Self
    where
        T: Into<TableRef>,
    {
        self.table = Some(table.into());
        self
    }

    /// Add a column to the index.
    pub fn column<T>(mut self, column: T) -> Self
    where
        T: IntoIden,
    {
        self.columns.push(column.into_iden());
        self
    }

    /// Set index as primary
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Set index as unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }
}

fn write_create_index<W: SqlWriter>(w: &mut W, index: &CreateIndex) {
    w.push_str("CREATE INDEX ");
    if index.primary {
        w.push_str("PRIMARY KEY ");
    }
    if index.unique {
        w.push_str("UNIQUE ");
    }
    w.push_str("ON ");
    if let Some(table) = &index.table {
        write_table_ref(w, table);
    }
    write_index_columns(w, &index.columns);
}

/// Write table index definition inside CREATE TABLE statement.
pub(crate) fn write_table_index<W: SqlWriter>(w: &mut W, index: &CreateIndex) {
    if index.primary {
        w.push_str("PRIMARY KEY ");
    }
    if index.unique {
        w.push_str("UNIQUE ");
    }
    write_index_columns(w, &index.columns);
}

fn write_index_columns<W: SqlWriter>(w: &mut W, columns: &[Iden]) {
    w.push_str("(");
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_iden(w, col);
    }
    w.push_str(")");
}
