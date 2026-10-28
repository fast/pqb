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
use crate::index::CreateIndex;
use crate::index::write_table_index;
use crate::table::ColumnDef;
use crate::table::PartitionBound;
use crate::table::PartitionBy;
use crate::table::TableConstraint;
use crate::table::write_column_spec;
use crate::table::write_column_type;
use crate::table::write_constraint;
use crate::table::write_partition_bound;
use crate::table::write_partition_by;
use crate::types::IntoTableRef;
use crate::types::TableName;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_name;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// Create a table.
#[derive(Default, Debug, Clone)]
pub struct CreateTable {
    table: Option<TableRef>,
    columns: Vec<ColumnDef>,
    indexes: Vec<CreateIndex>,
    constraints: Vec<TableConstraint>,
    if_not_exists: bool,
    temporary: bool,
    partition_by: Option<PartitionBy>,
    partition_of: Option<(TableName, PartitionBound)>,
}

impl CreateTable {
    /// Create a new CREATE TABLE statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render DDL with inline literals and no parameters. PostgreSQL utility statements do not
    /// accept bind parameters.
    pub fn to_values(&self) -> SqlWriterValues {
        SqlWriterValues::from_sql(self.to_sql())
    }

    /// Convert the CREATE TABLE statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_create_table(&mut sql, self);
        sql
    }

    /// Create table if table not exists.
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Create temporary table
    pub fn temporary(mut self) -> Self {
        self.temporary = true;
        self
    }

    /// Set table name.
    pub fn table<T>(mut self, table: T) -> Self
    where
        T: IntoTableRef,
    {
        self.table = Some(table.into_table_ref());
        self
    }

    /// Add a new table column.
    pub fn column(mut self, column: ColumnDef) -> Self {
        self.columns.push(column);
        self
    }

    /// Add a primary key, unique, check or foreign key constraint.
    pub fn constraint(mut self, constraint: impl Into<TableConstraint>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    /// Add a primary key index.
    pub fn primary_key(mut self, index: CreateIndex) -> Self {
        self.indexes.push(index.primary());
        self
    }

    /// Partition this table by range, list or hash keys. Also supports subpartitioning.
    pub fn partition_by(mut self, partition: PartitionBy) -> Self {
        self.partition_by = Some(partition);
        self
    }

    /// Create a partition of a parent table with inherited columns.
    ///
    /// Optional `column` definitions may override defaults or constraints, but must omit types.
    pub fn partition_of(mut self, parent: impl Into<TableName>, bound: PartitionBound) -> Self {
        self.partition_of = Some((parent.into(), bound));
        self
    }
}

fn write_create_table<W: SqlWriter>(w: &mut W, table: &CreateTable) {
    assert!(
        matches!(&table.table, Some(TableRef::Table(_, None))),
        "CREATE TABLE requires an unaliased table name"
    );
    if table.partition_of.is_some() {
        assert!(
            table.columns.iter().all(|column| column.ty.is_none()),
            "partition columns inherit their data types"
        );
    } else {
        assert!(
            table.columns.iter().all(|column| column.ty.is_some()),
            "CREATE TABLE columns require data types"
        );
    }
    w.push_str("CREATE ");
    if table.temporary {
        w.push_str("TEMPORARY ");
    }
    w.push_str("TABLE ");
    if table.if_not_exists {
        w.push_str("IF NOT EXISTS ");
    }
    if let Some(table_ref) = &table.table {
        write_table_ref(w, table_ref);
    }

    if let Some((parent, _)) = &table.partition_of {
        w.push_str(" PARTITION OF ");
        write_table_name(w, parent);
    }
    if table.partition_of.is_none()
        || !table.columns.is_empty()
        || !table.indexes.is_empty()
        || !table.constraints.is_empty()
    {
        w.push_str(" ( ");
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
        for col in &table.columns {
            write_comma_if_not_first!();
            write_iden(w, &col.name);
            if let Some(ty) = &col.ty {
                w.push_str(" ");
                write_column_type(w, ty);
            }
            write_column_spec(w, &col.spec);
        }

        for idx in &table.indexes {
            write_comma_if_not_first!();
            write_table_index(w, idx);
        }
        for constraint in &table.constraints {
            write_comma_if_not_first!();
            write_constraint(w, constraint);
        }
        let _ = is_first;
        w.push_str(" )");
    }
    if let Some((_, bound)) = &table.partition_of {
        w.push_char(' ');
        write_partition_bound(w, bound);
    }
    if let Some(partition) = &table.partition_by {
        write_partition_by(w, partition);
    }
}
