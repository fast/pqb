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
use crate::index::CreateIndex;
use crate::index::write_table_index;
use crate::table::ColumnDef;
use crate::table::write_column_spec;
use crate::table::write_column_type;
use crate::types::IntoTableRef;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// Create a table.
#[derive(Default, Debug, Clone)]
pub struct CreateTable {
    table: Option<TableRef>,
    columns: Vec<ColumnDef>,
    indexes: Vec<CreateIndex>,
    if_not_exists: bool,
    temporary: bool,
}

impl CreateTable {
    /// Create a new CREATE TABLE statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_create_table(&mut w, self);
        w
    }

    /// Convert the CREATE TABLE statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_create_table(&mut sql, self);
        sql
    }

    /// Create table if table not exists.
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Create temporary table
    pub fn temporary(mut self) -> Self {
        self.temporary = true;
        self
    }

    /// Set table name.
    pub fn table<T>(mut self, table: T) -> Self
    where
        T: IntoTableRef,
    {
        self.table = Some(table.into_table_ref());
        self
    }

    /// Add a new table column.
    pub fn column(mut self, column: ColumnDef) -> Self {
        self.columns.push(column);
        self
    }

    /// Add a primary key index.
    pub fn primary_key(mut self, index: CreateIndex) -> Self {
        self.indexes.push(index.primary());
        self
    }
}

fn write_create_table<W: SqlWriter>(w: &mut W, table: &CreateTable) {
    w.push_str("CREATE ");
    if table.temporary {
        w.push_str("TEMPORARY ");
    }
    w.push_str("TABLE ");
    if table.if_not_exists {
        w.push_str("IF NOT EXISTS ");
    }
    if let Some(table_ref) = &table.table {
        write_table_ref(w, table_ref);
    }

    w.push_str(" ( ");
    let mut is_first = true;
    macro_rules! write_comma_if_not_first {
        () => {
            if is_first {
                is_first = false
            } else {
                w.push_str(", ");
            }
        };
    }
    for col in &table.columns {
        write_comma_if_not_first!();
        write_iden(w, &col.name);
        if let Some(ty) = &col.ty {
            w.push_str(" ");
            write_column_type(w, ty);
        }
        write_column_spec(w, &col.spec);
    }

    for idx in &table.indexes {
        write_comma_if_not_first!();
        write_table_index(w, idx);
    }
    let _ = is_first;
    w.push_str(" )");
}
