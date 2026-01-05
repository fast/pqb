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
use crate::types::Iden;
use crate::types::TableRef;

/// Select rows from an existing table.
pub struct SelectStatement {
    selects: Vec<SelectExpr>,
    from: Vec<TableRef>,
}

impl SelectStatement {
    /// Convert the select statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let _ = self.selects.as_slice();
        let _ = self.from.as_slice();
        "SELECT 1".to_string()
    }
}

impl SelectStatement {
    pub(super) fn new() -> Self {
        Self {
            selects: vec![],
            from: vec![],
        }
    }
}

/// Select expression used in select statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectExpr {
    expr: Expr,
    alias: Option<Iden>,
}
