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

use crate::query::Select;
use crate::query::write_select;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::write_iden;
use crate::value::Value;
use crate::value::write_value;

/// A WITH clause can contain one or multiple common table expressions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct With {
    ctes: Vec<CommonTableExpression>,
}

impl With {
    /// Constructs a new WITH Clause.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a common table expression to this with clause.
    pub fn cte(mut self, cte: CommonTableExpression) -> Self {
        self.ctes.push(cte);
        self
    }
}

/// A table definition inside a WITH clause
#[derive(Debug, Clone, PartialEq)]
pub struct CommonTableExpression {
    name: Iden,
    columns: Vec<Iden>,
    query: Query,
    materialized: Option<bool>,
}

impl CommonTableExpression {
    /// Creates a new common table expression with the given name.
    pub fn new<T>(name: T) -> Self
    where
        T: IntoIden,
    {
        Self {
            name: name.into_iden(),
            columns: Vec::new(),
            query: Query::Values(vec![]),
            materialized: None,
        }
    }

    /// Sets the CTE VALUES source.
    pub fn values(mut self, values: Vec<Vec<Value>>) -> Self {
        self.query = Query::Values(values);
        self
    }

    /// Sets the CTE SELECT source.
    pub fn select(mut self, select: Select) -> Self {
        self.query = Query::Select(Box::new(select));
        self
    }

    /// Adds a named column to the CTE table definition.
    pub fn column<C>(mut self, col: C) -> Self
    where
        C: IntoIden,
    {
        self.columns.push(col.into_iden());
        self
    }

    /// Adds named columns to the CTE table definition.
    pub fn columns<T, I>(mut self, cols: I) -> Self
    where
        T: IntoIden,
        I: IntoIterator<Item = T>,
    {
        self.columns
            .extend(cols.into_iter().map(|col| col.into_iden()));
        self
    }

    /// Whether the CTE is materialized or not.
    pub fn materialized(mut self, materialized: bool) -> Self {
        self.materialized = Some(materialized);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Query {
    Select(Box<Select>),
    Values(Vec<Vec<Value>>),
}

pub(crate) fn write_with<W: crate::writer::SqlWriter>(w: &mut W, with: &With) {
    w.push_str("WITH ");
    for (i, cte) in with.ctes.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_iden(w, &cte.name);
        if cte.columns.is_empty() {
            w.push_str(" ");
        } else {
            w.push_str(" (");
            for (i, col) in cte.columns.iter().enumerate() {
                if i > 0 {
                    w.push_str(", ");
                }
                write_iden(w, col);
            }
            w.push_str(") ");
        }
        if let Some(materialized) = cte.materialized {
            if materialized {
                w.push_str(" AS MATERIALIZED ");
            } else {
                w.push_str(" AS NOT MATERIALIZED ");
            }
        } else {
            w.push_str(" AS ");
        }
        match &cte.query {
            Query::Select(select) => {
                w.push_char('(');
                write_select(w, select);
                w.push_char(')');
            }
            Query::Values(values) => {
                w.push_str("VALUES ");
                for (j, row) in values.iter().enumerate() {
                    if j > 0 {
                        w.push_str(", ");
                    }
                    w.push_char('(');
                    for (k, val) in row.iter().enumerate() {
                        if k > 0 {
                            w.push_str(", ");
                        }
                        write_value(w, val);
                    }
                    w.push_char(')');
                }
            }
        }
    }
}
