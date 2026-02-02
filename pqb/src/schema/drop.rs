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
use crate::types::SchemaName;
use crate::types::write_schema_name;
use crate::writer::SqlWriter;

/// DROP SCHEMA statement builder.
#[derive(Default, Debug, Clone)]
pub struct DropSchema {
    schemas: Vec<SchemaName>,
    if_exists: bool,
    behavior: Option<DropBehavior>,
}

impl DropSchema {
    /// Create a new DROP SCHEMA statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_drop_schema(&mut w, self);
        w
    }

    /// Convert the DROP SCHEMA statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_drop_schema(&mut sql, self);
        sql
    }

    /// Add a schema name to drop.
    pub fn schema<S>(mut self, schema: S) -> Self
    where
        S: Into<SchemaName>,
    {
        self.schemas.push(schema.into());
        self
    }

    /// Add multiple schema names to drop.
    pub fn schemas<I, S>(mut self, schemas: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SchemaName>,
    {
        self.schemas.extend(schemas.into_iter().map(Into::into));
        self
    }

    /// Drop the schema if it exists.
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

fn write_drop_schema<W: SqlWriter>(w: &mut W, drop_schema: &DropSchema) {
    w.push_str("DROP SCHEMA ");
    if drop_schema.if_exists {
        w.push_str("IF EXISTS ");
    }
    for (i, schema) in drop_schema.schemas.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_schema_name(w, schema);
    }
    if let Some(behavior) = drop_schema.behavior {
        w.push_char(' ');
        match behavior {
            DropBehavior::Cascade => w.push_str("CASCADE"),
            DropBehavior::Restrict => w.push_str("RESTRICT"),
        }
    }
}
