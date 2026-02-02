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

/// DROP INDEX statement builder.
#[derive(Default, Debug, Clone)]
pub struct DropIndex {
    indexes: Vec<TableName>,
    concurrently: bool,
    if_exists: bool,
    behavior: Option<DropBehavior>,
}

impl DropIndex {
    /// Create a new DROP INDEX statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_drop_index(&mut w, self);
        w
    }

    /// Convert the DROP INDEX statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_drop_index(&mut sql, self);
        sql
    }

    /// Add an index name to drop.
    pub fn index<N>(mut self, index: N) -> Self
    where
        N: Into<TableName>,
    {
        self.indexes.push(index.into());
        self
    }

    /// Add multiple index names to drop.
    pub fn indexes<I, N>(mut self, indexes: I) -> Self
    where
        I: IntoIterator<Item = N>,
        N: Into<TableName>,
    {
        self.indexes.extend(indexes.into_iter().map(Into::into));
        self
    }

    /// Drop the index if it exists.
    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }

    /// Drop the index concurrently.
    pub fn concurrently(mut self) -> Self {
        self.concurrently = true;
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

fn write_drop_index<W: SqlWriter>(w: &mut W, drop_index: &DropIndex) {
    w.push_str("DROP INDEX ");
    if drop_index.concurrently {
        w.push_str("CONCURRENTLY ");
    }
    if drop_index.if_exists {
        w.push_str("IF EXISTS ");
    }
    for (i, index) in drop_index.indexes.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_table_name(w, index);
    }
    if let Some(behavior) = drop_index.behavior {
        w.push_char(' ');
        match behavior {
            DropBehavior::Cascade => w.push_str("CASCADE"),
            DropBehavior::Restrict => w.push_str("RESTRICT"),
        }
    }
}
