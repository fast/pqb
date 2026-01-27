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

use crate::query::Select;
use crate::query::write_select;
use crate::writer::SqlWriter;

mod qualification;

/// An identifier string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Iden {
    name: Cow<'static, str>,
    // if true, the identifier needs not to be escaped when rendered
    escaped: bool,
}

impl Iden {
    /// Create a new identifier from a static str.
    pub const fn new_static(name: &'static str) -> Self {
        let escaped = is_escaped_iden(name);
        Self {
            name: Cow::Borrowed(name),
            escaped,
        }
    }

    /// Create a new identifier.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        let escaped = is_escaped_iden(&name);
        Self { name, escaped }
    }
}

/// Return whether this identifier needs to be escaped.
///
/// Right now this is an overrestricted check that only return `true` for identifiers composed of
/// `a-zA-Z0-9_`.
const fn is_escaped_iden(string: &str) -> bool {
    let bytes = string.as_bytes();
    if bytes.is_empty() {
        return true;
    }

    // can only begin with [a-z_]
    if bytes[0] == b'_' || (bytes[0] as char).is_ascii_alphabetic() {
        // good
    } else {
        return false;
    }

    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == b'_' || (bytes[i] as char).is_ascii_alphanumeric() {
            // good
        } else {
            return false;
        }
        i += 1;
    }

    true
}

impl From<&'static str> for Iden {
    fn from(name: &'static str) -> Self {
        Iden::new(name)
    }
}

impl From<String> for Iden {
    fn from(name: String) -> Self {
        Iden::new(name)
    }
}

impl From<Cow<'static, str>> for Iden {
    fn from(name: Cow<'static, str>) -> Self {
        Iden::new(name)
    }
}

/// A trait for types that can be converted into an identifier.
pub trait IntoIden {
    /// Convert into an identifier.
    fn into_iden(self) -> Iden;
}

impl<T> IntoIden for T
where
    T: Into<Iden>,
{
    fn into_iden(self) -> Iden {
        self.into()
    }
}

/// Asterisk ("*")
///
/// Express the asterisk without table prefix.
#[derive(Default, Debug, Clone, Copy)]
pub struct Asterisk;

/// Table references
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TableRef {
    /// A table identifier with optional Alias. Potentially qualified.
    Table(TableName, Option<Iden>),
    /// Subquery with alias
    SubQuery(Box<Select>, Iden),
}

impl TableRef {
    /// Add or replace the current alias
    pub fn alias<A>(self, alias: A) -> Self
    where
        A: IntoIden,
    {
        match self {
            Self::Table(table, _) => Self::Table(table, Some(alias.into_iden())),
            Self::SubQuery(statement, _) => Self::SubQuery(statement, alias.into_iden()),
        }
    }
}

impl<T> From<T> for TableRef
where
    T: Into<TableName>,
{
    fn from(value: T) -> Self {
        TableRef::Table(value.into(), None)
    }
}

/// A trait for types that can be converted into a table reference.
pub trait IntoTableRef: Into<TableRef> {
    /// Convert into a table reference.
    fn into_table_ref(self) -> TableRef {
        self.into()
    }
}

impl<T> IntoTableRef for T where T: Into<TableRef> {}

/// Column references.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ColumnRef {
    /// A column name, potentially qualified as `(database.)(schema.)(table.)column`.
    Column(ColumnName),
    /// An `*` expression, potentially qualified as `(database.)(schema.)(table.)*`.
    Asterisk(Option<TableName>),
}

impl From<Asterisk> for ColumnRef {
    fn from(_: Asterisk) -> Self {
        ColumnRef::Asterisk(None)
    }
}

impl<Table> From<(Table, Asterisk)> for ColumnRef
where
    Table: IntoIden,
{
    fn from(table: (Table, Asterisk)) -> Self {
        ColumnRef::Asterisk(Some(table.0.into()))
    }
}

impl<Schema, Table> From<(Schema, Table, Asterisk)> for ColumnRef
where
    Schema: IntoIden,
    Table: IntoIden,
{
    fn from(table: (Schema, Table, Asterisk)) -> Self {
        ColumnRef::Asterisk(Some((table.0, table.1).into()))
    }
}

impl<Database, Schema, Table> From<(Database, Schema, Table, Asterisk)> for ColumnRef
where
    Database: IntoIden,
    Schema: IntoIden,
    Table: IntoIden,
{
    fn from(table: (Database, Schema, Table, Asterisk)) -> Self {
        ColumnRef::Asterisk(Some((table.0, table.1, table.2).into()))
    }
}

impl<T> From<T> for ColumnRef
where
    T: Into<ColumnName>,
{
    fn from(value: T) -> Self {
        ColumnRef::Column(value.into())
    }
}

/// A trait for types that can be converted into a column reference.
pub trait IntoColumnRef: Into<ColumnRef> {
    /// Convert into a column reference.
    fn into_column_ref(self) -> ColumnRef {
        self.into()
    }
}

impl<T> IntoColumnRef for T where T: Into<ColumnRef> {}

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

/// Join types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum JoinType {
    LeftJoin,
    InnerJoin,
}

pub(crate) fn write_iden<W: SqlWriter>(w: &mut W, iden: &Iden) {
    // PostgreSQL uses double quotes for quoting identifiers.
    // @see https://www.postgresql.org/docs/18/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS
    const QUOTE: char = '"';

    w.push_char(QUOTE);
    if iden.escaped {
        w.push_str(&iden.name);
    } else {
        for ch in iden.name.chars() {
            // Escape quote characters by doubling them.
            if ch == QUOTE {
                w.push_char(QUOTE);
            }
            w.push_char(ch);
        }
    }
    w.push_char(QUOTE);
}

pub(crate) fn write_table_name<W: SqlWriter>(w: &mut W, table_name: &TableName) {
    let TableName(schema_name, table) = table_name;
    if let Some(schema_name) = schema_name {
        write_schema_name(w, schema_name);
        w.push_char('.');
    }
    write_iden(w, table);
}

pub(crate) fn write_schema_name<W: SqlWriter>(w: &mut W, schema_name: &SchemaName) {
    let SchemaName(database_name, schema) = schema_name;
    if let Some(DatabaseName(database)) = database_name {
        write_iden(w, database);
        w.push_char('.');
    }
    write_iden(w, schema);
}

pub(crate) fn write_table_ref<W: SqlWriter>(w: &mut W, table_ref: &TableRef) {
    match table_ref {
        TableRef::Table(table_name, alias) => {
            write_table_name(w, table_name);
            if let Some(alias) = alias {
                w.push_str(" AS ");
                write_iden(w, alias);
            }
        }
        TableRef::SubQuery(query, alias) => {
            w.push_char('(');
            write_select(w, query);
            w.push_char(')');
            w.push_str(" AS ");
            write_iden(w, alias);
        }
    }
}
