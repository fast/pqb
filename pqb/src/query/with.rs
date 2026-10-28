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
use crate::query::Statement;
use crate::query::write_statement;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::write_iden;
use crate::value::Value;

/// A WITH clause can contain one or multiple common table expressions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct With {
    ctes: Vec<CommonTableExpression>,
    recursive: bool,
}

impl With {
    /// Constructs a new WITH Clause.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow CTEs to reference themselves using WITH RECURSIVE.
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
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
        assert!(
            !values.is_empty() && !values[0].is_empty(),
            "CTE VALUES requires nonempty rows"
        );
        assert!(
            values.iter().all(|row| row.len() == values[0].len()),
            "CTE VALUES rows must have equal widths"
        );
        self.query = Query::Values(values);
        self
    }

    /// Sets the CTE SELECT source.
    pub fn select(self, select: Select) -> Self {
        self.query(select)
    }

    /// Set a SELECT, INSERT, UPDATE or DELETE source.
    ///
    /// A data-modifying CTE needs RETURNING if its output is referenced by the outer query.
    pub fn query(mut self, query: impl Into<Statement>) -> Self {
        self.query = Query::Statement(Box::new(query.into()));
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
    Statement(Box<Statement>),
    Values(Vec<Vec<Value>>),
}

pub(crate) fn write_with<W: crate::writer::SqlWriter>(w: &mut W, with: &With) {
    if with.ctes.is_empty() {
        return;
    }
    w.push_str("WITH ");
    if with.recursive {
        w.push_str("RECURSIVE ");
    }
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
        w.push_str("AS ");
        if let Some(materialized) = cte.materialized {
            if materialized {
                w.push_str("MATERIALIZED ");
            } else {
                w.push_str("NOT MATERIALIZED ");
            }
        }
        match &cte.query {
            Query::Statement(query) => {
                w.push_char('(');
                write_statement(w, query);
                w.push_char(')');
            }
            Query::Values(values) => {
                assert!(!values.is_empty(), "CTE requires a query or VALUES source");
                w.push_str("(VALUES ");
                for (j, row) in values.iter().enumerate() {
                    if j > 0 {
                        w.push_str(", ");
                    }
                    w.push_char('(');
                    for (k, val) in row.iter().enumerate() {
                        if k > 0 {
                            w.push_str(", ");
                        }
                        w.push_param(val.clone());
                    }
                    w.push_char(')');
                }
                w.push_char(')');
            }
        }
    }
}
