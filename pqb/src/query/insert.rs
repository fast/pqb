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

use crate::expr::Expr;
use crate::query::Returning;
use crate::types::{Iden, IntoIden, IntoTableRef, TableRef};

/// Insert any new rows into an existing table
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Insert {
    table: Option<TableRef>,
    columns: Vec<Iden>,
    returning: Option<Returning>,
}

impl Insert {
    /// Create a new INSERT statement.
    pub fn new() -> Self {
        Self::default()
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

    /// Specify a row of values to be inserted.
    pub fn values<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Expr>,
    {
        self
    }
}
