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
use crate::types::IntoColumnRef;
use crate::writer::SqlWriter;

/// RETURNING clause.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum Returning {
    All,
    Exprs(Vec<Expr>),
}

impl Returning {
    /// Create a RETURNING * clause.
    pub fn all() -> Self {
        Self::All
    }

    /// Create a RETURNING clause with a specific column.
    pub fn column<T>(col: T) -> Self
    where
        T: IntoColumnRef,
    {
        Self::Exprs(vec![Expr::column(col)])
    }

    /// Create a RETURNING clause with specific columns.
    pub fn columns<T, I>(cols: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoColumnRef,
    {
        Self::Exprs(cols.into_iter().map(Expr::column).collect())
    }

    /// Create a RETURNING clause with specific expressions.
    pub fn exprs<T, I>(exprs: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Expr>,
    {
        Self::Exprs(exprs.into_iter().map(Into::into).collect())
    }
}

pub(crate) fn write_returning<W: SqlWriter>(w: &mut W, returning: &Returning) {
    w.push_str(" RETURNING ");
    match returning {
        Returning::All => w.push_char('*'),
        Returning::Exprs(exprs) => {
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    w.push_str(", ");
                }
                write_expr(w, expr);
            }
        }
    }
}
