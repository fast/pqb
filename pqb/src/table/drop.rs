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
use crate::types::DropBehavior;
use crate::types::TableName;
use crate::types::write_table_name;
use crate::writer::SqlWriter;

/// DROP TABLE statement builder.
#[derive(Default, Debug, Clone)]
pub struct DropTable {
    tables: Vec<TableName>,
    if_exists: bool,
    behavior: Option<DropBehavior>,
}

impl DropTable {
    /// Create a new DROP TABLE statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_drop_table(&mut w, self);
        w
    }

    /// Convert the DROP TABLE statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_drop_table(&mut sql, self);
        sql
    }

    /// Add a table name to drop.
    pub fn table<T>(mut self, table: T) -> Self
    where
        T: Into<TableName>,
    {
        self.tables.push(table.into());
        self
    }

    /// Add multiple table names to drop.
    pub fn tables<I, T>(mut self, tables: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<TableName>,
    {
        self.tables.extend(tables.into_iter().map(Into::into));
        self
    }

    /// Drop the table if it exists.
    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }

    /// Add CASCADE to drop dependent objects.
    pub fn cascade(mut self) -> Self {
        self.behavior = Some(DropBehavior::Cascade);
        self
    }

    /// Add RESTRICT to drop (explicitly).
    pub fn restrict(mut self) -> Self {
        self.behavior = Some(DropBehavior::Restrict);
        self
    }
}

fn write_drop_table<W: SqlWriter>(w: &mut W, drop_table: &DropTable) {
    w.push_str("DROP TABLE ");
    if drop_table.if_exists {
        w.push_str("IF EXISTS ");
    }
    for (i, table) in drop_table.tables.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_table_name(w, table);
    }
    if let Some(behavior) = drop_table.behavior {
        w.push_char(' ');
        match behavior {
            DropBehavior::Cascade => w.push_str("CASCADE"),
            DropBehavior::Restrict => w.push_str("RESTRICT"),
        }
    }
}
