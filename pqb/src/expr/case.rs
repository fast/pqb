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
use crate::expr::write_expr;
use crate::writer::SqlWriter;

/// A searched CASE expression, evaluated in WHEN order.
#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    branches: Vec<(Expr, Expr)>,
    otherwise: Option<Box<Expr>>,
}

impl Case {
    /// Create CASE with its first WHEN condition and result.
    pub fn new(condition: impl Into<Expr>, result: impl Into<Expr>) -> Self {
        Self {
            branches: vec![(condition.into(), result.into())],
            otherwise: None,
        }
    }

    /// Append a WHEN branch.
    pub fn when(mut self, condition: impl Into<Expr>, result: impl Into<Expr>) -> Self {
        self.branches.push((condition.into(), result.into()));
        self
    }

    /// Set ELSE. If omitted, unmatched rows produce SQL NULL.
    pub fn otherwise(mut self, result: impl Into<Expr>) -> Self {
        self.otherwise = Some(Box::new(result.into()));
        self
    }
}

impl From<Case> for Expr {
    fn from(case: Case) -> Self {
        Self::Case(case)
    }
}

pub(super) fn write_case<W: SqlWriter>(w: &mut W, case: &Case) {
    w.push_str("CASE");
    for (condition, result) in &case.branches {
        w.push_str(" WHEN ");
        write_expr(w, condition);
        w.push_str(" THEN ");
        write_expr(w, result);
    }
    if let Some(result) = &case.otherwise {
        w.push_str(" ELSE ");
        write_expr(w, result);
    }
    w.push_str(" END");
}
