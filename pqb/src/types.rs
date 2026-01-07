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

//! Base types used throughout pqb.

use std::borrow::Cow;

/// An identifier string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Iden {
    name: Cow<'static, str>,
}

impl Iden {
    /// Create a new identifier.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { name: name.into() }
    }
}

/// Table references
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TableRef {
    /// A table identifier with optional Alias. Potentially qualified.
    Table(TableName, Option<Iden>),
}

/// Column references.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ColumnRef {
    /// A column name, potentially qualified as `(database.)(schema.)(table.)column`.
    Column(ColumnName),
    /// An `*` expression, potentially qualified as `(database.)(schema.)(table.)*`.
    Asterisk(Option<TableName>),
}

/// An identifier that represents a database name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatabaseName(pub Iden);

/// A schema name, potentially qualified as `(database.)schema`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaName(pub Option<DatabaseName>, pub Iden);

/// A table name, potentially qualified as `(database.)(schema.)table`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableName(pub Option<SchemaName>, pub Iden);

/// A column name, potentially qualified as `(database.)(schema.)(table.)column`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnName(pub Option<TableName>, pub Iden);
