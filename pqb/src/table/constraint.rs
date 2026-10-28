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
use crate::types::TableName;
use crate::types::write_iden;
use crate::types::write_table_name;
use crate::writer::SqlWriter;

/// A named or unnamed table constraint.
#[derive(Debug, Clone)]
pub struct TableConstraint {
    name: Option<Iden>,
    kind: ConstraintKind,
}

#[derive(Debug, Clone)]
enum ConstraintKind {
    PrimaryKey(Vec<Iden>),
    Unique(Vec<Iden>),
    Check(Expr),
    ForeignKey(ForeignKey),
}

impl TableConstraint {
    /// Create a primary key across one or more columns. Panics for an empty column list.
    pub fn primary_key<I, C>(columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoIden,
    {
        Self {
            name: None,
            kind: ConstraintKind::PrimaryKey(column_names(columns)),
        }
    }

    /// Create a unique constraint across one or more columns. Panics for an empty column list.
    pub fn unique<I, C>(columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoIden,
    {
        Self {
            name: None,
            kind: ConstraintKind::Unique(column_names(columns)),
        }
    }

    /// Require a check expression to be true or null for each row.
    pub fn check(expr: impl Into<Expr>) -> Self {
        Self {
            name: None,
            kind: ConstraintKind::Check(expr.into()),
        }
    }

    /// Name this constraint, quoting the identifier when rendered.
    pub fn name(mut self, name: impl IntoIden) -> Self {
        self.name = Some(name.into_iden());
        self
    }
}

/// Referential actions for foreign keys.
#[derive(Debug, Clone, Copy)]
#[expect(missing_docs)]
pub enum ReferentialAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

/// A foreign key referencing columns of another table.
#[derive(Debug, Clone)]
pub struct ForeignKey {
    columns: Vec<Iden>,
    table: TableName,
    referenced: Vec<Iden>,
    on_delete: Option<ReferentialAction>,
    on_update: Option<ReferentialAction>,
}

impl ForeignKey {
    /// Reference equally sized, nonempty column lists. Panics if their sizes differ or either is
    /// empty.
    pub fn new<I, C, J, R>(columns: I, table: impl Into<TableName>, referenced: J) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoIden,
        J: IntoIterator<Item = R>,
        R: IntoIden,
    {
        let columns = column_names(columns);
        let referenced = column_names(referenced);
        assert_eq!(
            columns.len(),
            referenced.len(),
            "foreign key column counts must match"
        );
        Self {
            columns,
            table: table.into(),
            referenced,
            on_delete: None,
            on_update: None,
        }
    }

    /// Set the action taken when a referenced row is deleted.
    pub fn on_delete(mut self, action: ReferentialAction) -> Self {
        self.on_delete = Some(action);
        self
    }

    /// Set the action taken when a referenced key is updated.
    pub fn on_update(mut self, action: ReferentialAction) -> Self {
        self.on_update = Some(action);
        self
    }

    /// Name this foreign key constraint.
    pub fn name(self, name: impl IntoIden) -> TableConstraint {
        TableConstraint::from(self).name(name)
    }
}

impl From<ForeignKey> for TableConstraint {
    fn from(key: ForeignKey) -> Self {
        Self {
            name: None,
            kind: ConstraintKind::ForeignKey(key),
        }
    }
}

fn column_names<I, C>(columns: I) -> Vec<Iden>
where
    I: IntoIterator<Item = C>,
    C: IntoIden,
{
    let columns: Vec<_> = columns.into_iter().map(IntoIden::into_iden).collect();
    assert!(
        !columns.is_empty(),
        "constraint requires at least one column"
    );
    columns
}

fn write_columns<W: SqlWriter>(w: &mut W, columns: &[Iden]) {
    w.push_char('(');
    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_iden(w, column);
    }
    w.push_char(')');
}

pub(crate) fn write_constraint<W: SqlWriter>(w: &mut W, constraint: &TableConstraint) {
    if let Some(name) = &constraint.name {
        w.push_str("CONSTRAINT ");
        write_iden(w, name);
        w.push_char(' ');
    }
    match &constraint.kind {
        ConstraintKind::PrimaryKey(columns) => {
            w.push_str("PRIMARY KEY ");
            write_columns(w, columns);
        }
        ConstraintKind::Unique(columns) => {
            w.push_str("UNIQUE ");
            write_columns(w, columns);
        }
        ConstraintKind::Check(expr) => {
            w.push_str("CHECK (");
            write_expr(w, expr);
            w.push_char(')');
        }
        ConstraintKind::ForeignKey(key) => {
            w.push_str("FOREIGN KEY ");
            write_columns(w, &key.columns);
            w.push_str(" REFERENCES ");
            write_table_name(w, &key.table);
            w.push_char(' ');
            write_columns(w, &key.referenced);
            for (prefix, action) in [
                (" ON DELETE ", key.on_delete),
                (" ON UPDATE ", key.on_update),
            ] {
                if let Some(action) = action {
                    w.push_str(prefix);
                    w.push_str(match action {
                        ReferentialAction::NoAction => "NO ACTION",
                        ReferentialAction::Restrict => "RESTRICT",
                        ReferentialAction::Cascade => "CASCADE",
                        ReferentialAction::SetNull => "SET NULL",
                        ReferentialAction::SetDefault => "SET DEFAULT",
                    });
                }
            }
        }
    }
}
