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
use crate::expr::Expr;
use crate::expr::write_expr;
use crate::query::Returning;
use crate::query::With;
use crate::query::write_returning;
use crate::query::write_with;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::IntoTableRef;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// Conditionally insert, update or delete rows using a source relation (PostgreSQL 15+).
///
/// WHEN clauses are evaluated in insertion order; the first matching clause wins.
/// RETURNING and NOT MATCHED BY SOURCE require PostgreSQL 17+.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Merge {
    table: Option<TableRef>,
    source: Option<TableRef>,
    on: Option<Box<Expr>>,
    clauses: Vec<MergeWhen>,
    with: Option<With>,
    pub(super) returning: Option<Returning>,
    only: bool,
}

impl Merge {
    /// Create a MERGE statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target table, optionally aliased with [`TableRef::alias`].
    pub fn into_table(mut self, table: impl IntoTableRef) -> Self {
        self.table = Some(table.into());
        self
    }

    /// Exclude inherited tables when matching target rows.
    pub fn only(mut self) -> Self {
        self.only = true;
        self
    }

    /// Set the source table or aliased SELECT subquery.
    pub fn using(mut self, source: impl IntoTableRef) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the condition joining the source and target.
    pub fn on(mut self, condition: impl Into<Expr>) -> Self {
        self.on = Some(Box::new(condition.into()));
        self
    }

    /// Append a WHEN clause. An unconditional clause must be last for its match kind.
    pub fn when(mut self, clause: MergeWhen) -> Self {
        assert!(
            !self
                .clauses
                .iter()
                .any(|previous| { previous.kind == clause.kind && previous.conditions.is_empty() }),
            "MERGE clause is unreachable after an unconditional clause of the same kind"
        );
        self.clauses.push(clause);
        self
    }

    /// Set common table expressions. MERGE does not support WITH RECURSIVE.
    pub fn with(mut self, with: With) -> Self {
        assert!(!with.recursive, "MERGE does not support WITH RECURSIVE");
        self.with = Some(with);
        self
    }

    /// Return modified rows (PostgreSQL 17+).
    pub fn returning(mut self, returning: Returning) -> Self {
        self.returning = Some(returning);
        self
    }

    /// Render inline literals.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_merge(&mut sql, self);
        sql
    }

    /// Render numbered placeholders and ordered parameters.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut writer = SqlWriterValues::new();
        write_merge(&mut writer, self);
        writer
    }
}

/// An action taken by a MERGE WHEN clause.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum MergeAction {
    Update(Vec<(Iden, Expr)>),
    Delete,
    Insert {
        columns: Vec<Iden>,
        values: Vec<Expr>,
    },
    DefaultValues,
    DoNothing,
}

impl MergeAction {
    /// Update one or more target columns.
    pub fn update<C: IntoIden>(values: impl IntoIterator<Item = (C, Expr)>) -> Self {
        Self::Update(
            values
                .into_iter()
                .map(|(c, v)| (c.into_iden(), v))
                .collect(),
        )
    }

    /// Insert a row. An empty column list uses the target table's column order.
    pub fn insert<C: IntoIden>(
        columns: impl IntoIterator<Item = C>,
        values: impl IntoIterator<Item = Expr>,
    ) -> Self {
        Self::Insert {
            columns: columns.into_iter().map(IntoIden::into_iden).collect(),
            values: values.into_iter().collect(),
        }
    }
}

/// A match kind, optional conditions and action in MERGE.
#[derive(Debug, Clone, PartialEq)]
pub struct MergeWhen {
    kind: MatchKind,
    conditions: Vec<Expr>,
    action: MergeAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchKind {
    Matched,
    NotMatched,
    NotMatchedBySource,
}

impl MergeWhen {
    /// Act on a target row with a matching source row.
    pub fn matched(action: MergeAction) -> Self {
        Self::new(MatchKind::Matched, action)
    }

    /// Act on a source row without a matching target row.
    pub fn not_matched(action: MergeAction) -> Self {
        Self::new(MatchKind::NotMatched, action)
    }

    /// Act on a target row absent from the source (PostgreSQL 17+).
    pub fn not_matched_by_source(action: MergeAction) -> Self {
        Self::new(MatchKind::NotMatchedBySource, action)
    }

    fn new(kind: MatchKind, action: MergeAction) -> Self {
        let valid = match &action {
            MergeAction::Insert { columns, values } => {
                assert!(!values.is_empty(), "MERGE INSERT requires a nonempty row");
                assert!(
                    columns.is_empty() || columns.len() == values.len(),
                    "MERGE INSERT column and value counts must match"
                );
                kind == MatchKind::NotMatched
            }
            MergeAction::DefaultValues => kind == MatchKind::NotMatched,
            MergeAction::Update(values) => {
                assert!(!values.is_empty(), "MERGE UPDATE requires assignments");
                kind != MatchKind::NotMatched
            }
            MergeAction::Delete => kind != MatchKind::NotMatched,
            MergeAction::DoNothing => true,
        };
        assert!(valid, "MERGE action is not allowed for this match kind");
        Self {
            kind,
            conditions: Vec::new(),
            action,
        }
    }

    /// Require an additional condition before taking this action.
    pub fn and_where(mut self, condition: impl Into<Expr>) -> Self {
        self.conditions.push(condition.into());
        self
    }
}

pub(crate) fn write_merge<W: SqlWriter>(w: &mut W, merge: &Merge) {
    assert!(
        matches!(&merge.table, Some(TableRef::Table(..))),
        "MERGE requires a target table"
    );
    assert!(
        matches!(
            &merge.source,
            Some(TableRef::Table(..) | TableRef::SubQuery(..))
        ),
        "MERGE requires a source table or SELECT subquery"
    );
    let on = merge.on.as_ref().expect("MERGE requires an ON condition");
    assert!(!merge.clauses.is_empty(), "MERGE requires WHEN clauses");
    if let Some(with) = &merge.with {
        write_with(w, with);
        w.push_char(' ');
    }
    w.push_str("MERGE INTO ");
    if merge.only {
        w.push_str("ONLY ");
    }
    write_table_ref(w, merge.table.as_ref().unwrap());
    w.push_str(" USING ");
    write_table_ref(w, merge.source.as_ref().unwrap());
    w.push_str(" ON ");
    write_expr(w, on);
    for clause in &merge.clauses {
        w.push_str(match clause.kind {
            MatchKind::Matched => " WHEN MATCHED",
            MatchKind::NotMatched => " WHEN NOT MATCHED",
            MatchKind::NotMatchedBySource => " WHEN NOT MATCHED BY SOURCE",
        });
        if let Some(condition) = Expr::from_conditions(clause.conditions.clone()) {
            w.push_str(" AND ");
            write_expr(w, &condition);
        }
        w.push_str(" THEN ");
        match &clause.action {
            MergeAction::Update(values) => {
                w.push_str("UPDATE SET ");
                for (i, (column, value)) in values.iter().enumerate() {
                    if i > 0 {
                        w.push_str(", ");
                    }
                    write_iden(w, column);
                    w.push_str(" = ");
                    write_expr(w, value);
                }
            }
            MergeAction::Delete => w.push_str("DELETE"),
            MergeAction::DoNothing => w.push_str("DO NOTHING"),
            MergeAction::DefaultValues => w.push_str("INSERT DEFAULT VALUES"),
            MergeAction::Insert { columns, values } => {
                w.push_str("INSERT");
                if !columns.is_empty() {
                    w.push_str(" (");
                    for (i, column) in columns.iter().enumerate() {
                        if i > 0 {
                            w.push_str(", ");
                        }
                        write_iden(w, column);
                    }
                    w.push_char(')');
                }
                w.push_str(" VALUES (");
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        w.push_str(", ");
                    }
                    write_expr(w, value);
                }
                w.push_char(')');
            }
        }
    }
    if let Some(returning) = &merge.returning {
        write_returning(w, returning);
    }
}
