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
use crate::expr::write_expr;
use crate::table::ColumnDef;
use crate::table::IntoColumnDef;
use crate::table::write_column_spec;
use crate::table::write_column_type;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::IntoTableRef;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// Alter table statement.
#[derive(Default, Debug, Clone)]
pub struct AlterTable {
    table: Option<TableRef>,
    options: Vec<TableAlterOption>,
}

impl AlterTable {
    /// Create a new ALTER TABLE statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_alter_table(&mut w, self);
        w
    }

    /// Convert the alter table statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_alter_table(&mut sql, self);
        sql
    }

    /// Set table name.
    pub fn table<R>(mut self, table: R) -> Self
    where
        R: IntoTableRef,
    {
        self.table = Some(table.into_table_ref());
        self
    }

    /// Add a column to an existing table
    pub fn add_column<C>(mut self, column_def: C) -> Self
    where
        C: IntoColumnDef,
    {
        self.options
            .push(TableAlterOption::AddColumn(AddColumnOption {
                column: column_def.into_column_def(),
                if_not_exists: false,
            }));
        self
    }

    /// Modify a column in an existing table
    pub fn modify_column<C>(mut self, column_def: C) -> Self
    where
        C: IntoColumnDef,
    {
        self.options
            .push(TableAlterOption::ModifyColumn(column_def.into_column_def()));
        self
    }

    /// Drop a column from an existing table
    pub fn drop_column<T>(mut self, col: T) -> Self
    where
        T: IntoIden,
    {
        self.options
            .push(TableAlterOption::DropColumn(DropColumnOption {
                column: col.into_iden(),
                if_exists: false,
            }));
        self
    }

    /// Rename a column in an existing table
    pub fn rename_column<T, R>(mut self, from_name: T, to_name: R) -> Self
    where
        T: IntoIden,
        R: IntoIden,
    {
        self.options.push(TableAlterOption::RenameColumn(
            from_name.into_iden(),
            to_name.into_iden(),
        ));
        self
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)] // add_foreign_key, drop_foreign_key, ...
enum TableAlterOption {
    AddColumn(AddColumnOption),
    ModifyColumn(ColumnDef),
    RenameColumn(Iden, Iden),
    DropColumn(DropColumnOption),
}

#[derive(Debug, Clone)]
struct AddColumnOption {
    column: ColumnDef,
    if_not_exists: bool,
}

#[derive(Debug, Clone)]
struct DropColumnOption {
    column: Iden,
    if_exists: bool,
}

fn write_alter_table<W: SqlWriter>(w: &mut W, alter: &AlterTable) {
    w.push_str("ALTER TABLE ");
    if let Some(table) = &alter.table {
        write_table_ref(w, table);
        w.push_char(' ');
    }
    for (i, option) in alter.options.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        match option {
            TableAlterOption::AddColumn(AddColumnOption {
                column,
                if_not_exists,
            }) => {
                w.push_str("ADD COLUMN ");
                if *if_not_exists {
                    w.push_str("IF NOT EXISTS ");
                }
                write_iden(w, &column.name);
                if let Some(ty) = &column.ty {
                    w.push_str(" ");
                    write_column_type(w, ty);
                }
                write_column_spec(w, &column.spec);
            }
            TableAlterOption::ModifyColumn(column_def) => {
                let mut is_first = true;

                macro_rules! write_comma_if_not_first {
                    () => {
                        if is_first {
                            is_first = false
                        } else {
                            w.push_str(", ");
                        }
                    };
                }

                if let Some(ty) = &column_def.ty {
                    write_comma_if_not_first!();
                    w.push_str("ALTER COLUMN ");
                    write_iden(w, &column_def.name);
                    w.push_str(" TYPE ");
                    write_column_type(w, ty);
                }

                if let Some(nullable) = column_def.spec.nullable {
                    write_comma_if_not_first!();
                    w.push_str("ALTER COLUMN ");
                    write_iden(w, &column_def.name);
                    if nullable {
                        w.push_str(" DROP NOT NULL");
                    } else {
                        w.push_str(" SET NOT NULL");
                    }
                }

                if let Some(default) = &column_def.spec.default {
                    write_comma_if_not_first!();
                    w.push_str("ALTER COLUMN ");
                    write_iden(w, &column_def.name);
                    w.push_str(" SET DEFAULT ");
                    write_expr(w, default);
                }

                if column_def.spec.unique {
                    write_comma_if_not_first!();
                    w.push_str("ADD UNIQUE (");
                    write_iden(w, &column_def.name);
                    w.push_str(")");
                }

                if column_def.spec.primary_key {
                    write_comma_if_not_first!();
                    w.push_str("ADD PRIMARY KEY (");
                    write_iden(w, &column_def.name);
                    w.push_str(")");
                }

                let _ = is_first;
            }
            TableAlterOption::RenameColumn(from, to) => {
                w.push_str("RENAME COLUMN ");
                write_iden(w, from);
                w.push_str(" TO ");
                write_iden(w, to);
            }
            TableAlterOption::DropColumn(DropColumnOption { column, if_exists }) => {
                w.push_str("DROP COLUMN ");
                if *if_exists {
                    w.push_str("IF EXISTS ");
                }
                write_iden(w, column);
            }
        }
    }
}
