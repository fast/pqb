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

// -------------------------- MaybeQualifiedOnce -------------------------------

use crate::types::ColumnName;
use crate::types::DatabaseName;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::SchemaName;
use crate::types::TableName;

/// A name that can be unqualified (`foo`) or qualified once (`foo.bar`).
///
/// This is mostly a "private" helper trait to provide reusable conversions.
pub trait MaybeQualifiedOnce {
    /// Represent a maybe-qualified name as a `(foo?, bar)` tuple.
    fn into_2_parts(self) -> (Option<Iden>, Iden);
}

/// Only the "base", no qualification (`foo`).
impl<T> MaybeQualifiedOnce for T
where
    T: IntoIden,
{
    fn into_2_parts(self) -> (Option<Iden>, Iden) {
        (None, self.into_iden())
    }
}

/// With a qualification (`foo.bar`).
impl<S, T> MaybeQualifiedOnce for (S, T)
where
    S: IntoIden,
    T: IntoIden,
{
    fn into_2_parts(self) -> (Option<Iden>, Iden) {
        let (qual, base) = self;
        (Some(qual.into_iden()), base.into_iden())
    }
}

// ------------------------- MaybeQualifiedTwice -------------------------------

/// A name that can be unqualified (`foo`), qualified once (`foo.bar`), or twice (`foo.bar.baz`).
///
/// This is mostly a "private" helper trait to provide reusable conversions.
pub trait MaybeQualifiedTwice {
    /// Represent a maybe-qualified name as a `(foo?, bar?, baz)` tuple.
    ///
    /// To be precise, it's actually `((foo?, bar)?, baz)` to rule out invalid states like `(Some,
    /// None, Some)`.
    fn into_3_parts(self) -> (Option<(Option<Iden>, Iden)>, Iden);
}

/// From 1 or 2 parts (`foo` or `foo.bar`).
impl<T> MaybeQualifiedTwice for T
where
    T: MaybeQualifiedOnce,
{
    fn into_3_parts(self) -> (Option<(Option<Iden>, Iden)>, Iden) {
        let (middle, base) = self.into_2_parts();
        let qual = middle.map(|middle| (None, middle));
        (qual, base)
    }
}

/// Fully-qualified from 3 parts (`foo.bar.baz`).
impl<S, T, U> MaybeQualifiedTwice for (S, T, U)
where
    S: IntoIden,
    T: IntoIden,
    U: IntoIden,
{
    fn into_3_parts(self) -> (Option<(Option<Iden>, Iden)>, Iden) {
        let (q2, q1, base) = self;
        let (q2, q1, base) = (q2.into_iden(), q1.into_iden(), base.into_iden());
        let q = (Some(q2), q1);
        (Some(q), base)
    }
}

// -------------------------------- impls --------------------------------------

/// Construct a [`SchemaName`] from 1-2 parts (`(database?).schema`)
impl<T> From<T> for SchemaName
where
    T: MaybeQualifiedOnce,
{
    fn from(value: T) -> Self {
        let (db, schema) = value.into_2_parts();
        let db_name = db.map(DatabaseName);
        SchemaName(db_name, schema)
    }
}

/// Construct a [`TableName`] from 1-3 parts (`(database?).(schema?).table`)
impl<T> From<T> for TableName
where
    T: MaybeQualifiedTwice,
{
    fn from(value: T) -> Self {
        let (schema_parts, table) = value.into_3_parts();
        let schema_name = schema_parts.map(|schema_parts| match schema_parts {
            (Some(db), schema) => SchemaName(Some(DatabaseName(db)), schema),
            (None, schema) => SchemaName(None, schema),
        });
        TableName(schema_name, table)
    }
}

/// Construct a [`ColumnName`] from 1-3 parts (`(schema?).(table?).column`)
impl<T> From<T> for ColumnName
where
    T: MaybeQualifiedTwice,
{
    fn from(value: T) -> Self {
        let (table_parts, column) = value.into_3_parts();
        let table_name = table_parts.map(|table_parts| match table_parts {
            (Some(schema), table) => TableName(Some(schema.into()), table),
            (None, table) => TableName(None, table),
        });
        ColumnName(table_name, column)
    }
}
