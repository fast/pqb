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

use std::borrow::Cow;

use crate::expr::Expr;
use crate::expr::write_expr;
use crate::writer::SqlWriter;

/// SQL syntax interleaved with expressions that retain normal parameter binding.
///
/// Only pass trusted syntax to `new` and `sql`. Pass user values through `expr`.
/// No placeholder scanning or substitution is performed on SQL text.
#[derive(Debug, Clone, PartialEq)]
pub struct SqlFragment {
    parts: Vec<Expr>,
}

impl SqlFragment {
    /// Start a fragment with trusted SQL syntax.
    pub fn new(sql: impl Into<Cow<'static, str>>) -> Self {
        Self {
            parts: vec![Expr::custom(sql)],
        }
    }

    /// Append trusted SQL syntax verbatim.
    pub fn sql(mut self, sql: impl Into<Cow<'static, str>>) -> Self {
        self.parts.push(Expr::custom(sql));
        self
    }

    /// Append an expression, binding values with the surrounding query's parameter numbering.
    ///
    /// Add parentheses with `sql` when embedding a compound expression in an operator context.
    pub fn expr(mut self, expr: impl Into<Expr>) -> Self {
        self.parts.push(expr.into());
        self
    }
}

impl From<SqlFragment> for Expr {
    fn from(fragment: SqlFragment) -> Self {
        Self::Fragment(fragment)
    }
}

pub(super) fn write_fragment<W: SqlWriter>(w: &mut W, fragment: &SqlFragment) {
    for part in &fragment.parts {
        write_expr(w, part);
    }
}
