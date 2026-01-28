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
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::write_iden;
use crate::writer::SqlWriter;

/// ON CONFLICT clause for INSERT statements.
#[derive(Debug, Clone, PartialEq)]
pub struct OnConflict {
    targets: OnConflictTarget,
    target_conditions: Vec<Expr>,
    action: Option<OnConflictAction>,
    action_conditions: Vec<Expr>,
}

impl Default for OnConflict {
    fn default() -> Self {
        OnConflict {
            targets: OnConflictTarget::Exprs(vec![]),
            target_conditions: vec![],
            action: None,
            action_conditions: vec![],
        }
    }
}

impl OnConflict {
    /// Create a new ON CONFLICT clause with empty targets.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set ON CONFLICT target column.
    pub fn column<C>(column: C) -> Self
    where
        C: IntoIden,
    {
        OnConflict::columns([column])
    }

    /// Set ON CONFLICT target columns.
    pub fn columns<I, C>(columns: I) -> Self
    where
        C: IntoIden,
        I: IntoIterator<Item = C>,
    {
        OnConflict {
            targets: OnConflictTarget::Exprs(columns.into_iter().map(Expr::column).collect()),
            target_conditions: vec![],
            action: None,
            action_conditions: vec![],
        }
    }

    /// Set ON CONSTRAINT target constraint name.
    pub fn constraint(constraint: impl Into<String>) -> Self {
        OnConflict {
            targets: OnConflictTarget::Constraint(constraint.into()),
            target_conditions: vec![],
            action: None,
            action_conditions: vec![],
        }
    }

    /// Set ON CONFLICT target expression.
    pub fn expr<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        OnConflict::exprs([expr])
    }

    /// Set multiple target expressions for ON CONFLICT.
    pub fn exprs<I, T>(exprs: I) -> Self
    where
        T: Into<Expr>,
        I: IntoIterator<Item = T>,
    {
        OnConflict {
            targets: OnConflictTarget::Exprs(exprs.into_iter().map(Into::into).collect()),
            target_conditions: vec![],
            action: None,
            action_conditions: vec![],
        }
    }

    /// Set ON CONFLICT do nothing.
    pub fn do_nothing(mut self) -> Self {
        self.action = Some(OnConflictAction::DoNothing);
        self
    }

    /// Set ON CONFLICT update column.
    pub fn update_column<C>(self, column: C) -> Self
    where
        C: IntoIden,
    {
        self.update_columns([column])
    }

    /// Set ON CONFLICT update columns.
    pub fn update_columns<C, I>(self, columns: I) -> Self
    where
        C: IntoIden,
        I: IntoIterator<Item = C>,
    {
        let updates = columns
            .into_iter()
            .map(|x| OnConflictUpdate::Column(IntoIden::into_iden(x)));
        self.updates(updates)
    }

    /// Set ON CONFLICT update value
    pub fn value<C, T>(self, col: C, value: T) -> Self
    where
        C: IntoIden,
        T: Into<Expr>,
    {
        self.values([(col, value.into())])
    }

    /// Set ON CONFLICT update exprs. Append to current list of expressions.
    pub fn values<C, I>(self, values: I) -> Self
    where
        C: IntoIden,
        I: IntoIterator<Item = (C, Expr)>,
    {
        let updates = values
            .into_iter()
            .map(|(c, e)| OnConflictUpdate::Expr(c.into_iden(), e));
        self.updates(updates)
    }

    /// Set target WHERE.
    pub fn target_and_where<T>(mut self, expr: T) -> Self
    where
        T: Into<Expr>,
    {
        self.target_conditions.push(expr.into());
        self
    }

    /// Set action WHERE.
    pub fn action_and_where<T>(mut self, expr: T) -> Self
    where
        T: Into<Expr>,
    {
        self.action_conditions.push(expr.into());
        self
    }
}

impl OnConflict {
    fn updates<I>(mut self, updates: I) -> Self
    where
        I: IntoIterator<Item = OnConflictUpdate>,
    {
        match &mut self.action {
            Some(OnConflictAction::Update(v)) => v.extend(updates),
            Some(OnConflictAction::DoNothing) | None => {
                self.action = Some(OnConflictAction::Update(updates.into_iter().collect()));
            }
        };
        self
    }
}

/// Represents ON CONFLICT (upsert) targets
///
/// Targets can be a list of columns or expressions, even mixed, or just a
/// single constraint name.
#[derive(Debug, Clone, PartialEq)]
enum OnConflictTarget {
    Exprs(Vec<Expr>),
    Constraint(String),
}

/// Represents ON CONFLICT (upsert) actions
#[derive(Debug, Clone, PartialEq)]
enum OnConflictAction {
    DoNothing,
    Update(Vec<OnConflictUpdate>),
}

/// Represents strategies to update column in ON CONFLICT (upsert) actions
#[derive(Debug, Clone, PartialEq)]
enum OnConflictUpdate {
    /// Update column value of existing row with inserting value
    Column(Iden),
    /// Update column value of existing row with expression
    Expr(Iden, Expr),
}

pub(crate) fn write_on_conflict<W: SqlWriter>(w: &mut W, on_conflict: &OnConflict) {
    w.push_str(" ON CONFLICT ");
    match &on_conflict.targets {
        OnConflictTarget::Exprs(exprs) => {
            w.push_str("(");
            for (i, expr) in exprs.iter().enumerate() {
                if i > 0 {
                    w.push_str(", ");
                }
                write_expr(w, expr);
            }
            w.push_str(")");
        }
        OnConflictTarget::Constraint(constraint) => {
            w.push_str("ON CONSTRAINT ");
            w.push_char('"');
            w.push_str(constraint);
            w.push_char('"');
        }
    }
    if let Some(condition) = Expr::from_conditions(on_conflict.target_conditions.clone()) {
        w.push_str(" WHERE ");
        write_expr(w, &condition);
    }
    if let Some(action) = &on_conflict.action {
        match action {
            OnConflictAction::DoNothing => w.push_str(" DO NOTHING"),
            OnConflictAction::Update(updates) => {
                w.push_str(" DO UPDATE SET ");
                for (i, update) in updates.iter().enumerate() {
                    if i > 0 {
                        w.push_str(", ");
                    }
                    match update {
                        OnConflictUpdate::Column(col) => {
                            write_iden(w, col);
                            w.push_str(r#" = "excluded"."#);
                            write_iden(w, col);
                        }
                        OnConflictUpdate::Expr(col, expr) => {
                            write_iden(w, col);
                            w.push_str(" = ");
                            write_expr(w, expr);
                        }
                    }
                }
            }
        }
    }
    if let Some(condition) = Expr::from_conditions(on_conflict.action_conditions.clone()) {
        w.push_str(" WHERE ");
        write_expr(w, &condition);
    }
}
