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
use crate::types::write_iden;

/// Create a schema, quoting its name as an identifier.
#[derive(Debug, Clone)]
pub struct CreateSchema {
    name: Iden,
    if_not_exists: bool,
}

impl CreateSchema {
    /// Create a schema with this name.
    pub fn new(name: impl IntoIden) -> Self {
        Self {
            name: name.into_iden(),
            if_not_exists: false,
        }
    }

    /// Leave an existing schema unchanged.
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Render the CREATE SCHEMA statement.
    pub fn to_sql(&self) -> String {
        let mut sql = String::from("CREATE SCHEMA ");
        if self.if_not_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        write_iden(&mut sql, &self.name);
        sql
    }

    /// Render DDL with no bind parameters.
    pub fn to_values(&self) -> SqlWriterValues {
        SqlWriterValues::from_sql(self.to_sql())
    }
}
