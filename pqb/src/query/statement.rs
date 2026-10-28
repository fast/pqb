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
use crate::query::Delete;
use crate::query::Insert;
use crate::query::Select;
use crate::query::Update;
use crate::query::write_delete;
use crate::query::write_insert;
use crate::query::write_select;
use crate::query::write_update;
use crate::writer::SqlWriter;

/// A data query usable as a top-level statement, CTE source or EXPLAIN input.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum Statement {
    Select(Select),
    Insert(Insert),
    Update(Update),
    Delete(Delete),
}

impl From<Select> for Statement {
    fn from(s: Select) -> Self {
        Statement::Select(s)
    }
}

impl From<Insert> for Statement {
    fn from(i: Insert) -> Self {
        Statement::Insert(i)
    }
}

impl From<Update> for Statement {
    fn from(u: Update) -> Self {
        Statement::Update(u)
    }
}

impl From<Delete> for Statement {
    fn from(d: Delete) -> Self {
        Statement::Delete(d)
    }
}

impl Statement {
    /// Render a query with inline literals.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_statement(&mut sql, self);
        sql
    }

    /// Render a query with numbered placeholders and ordered parameters.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut writer = SqlWriterValues::new();
        write_statement(&mut writer, self);
        writer
    }
}

pub(crate) fn write_statement<W: SqlWriter>(writer: &mut W, statement: &Statement) {
    match statement {
        Statement::Select(query) => write_select(writer, query),
        Statement::Insert(query) => write_insert(writer, query),
        Statement::Update(query) => write_update(writer, query),
        Statement::Delete(query) => write_delete(writer, query),
    }
}
