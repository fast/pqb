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

use std::borrow::Cow;

use crate::SqlWriterValues;
use crate::expr::Expr;
use crate::expr::write_expr;
use crate::expr::write_tuple;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;

/// CREATE INDEX statement builder.
#[derive(Default, Debug, Clone)]
pub struct CreateIndex {
    table: Option<TableRef>,
    concurrently: bool,
    if_not_exists: bool,
    primary: bool,
    unique: bool,
    name: Option<Iden>,
    columns: Vec<Expr>,
    include_columns: Vec<Iden>,
    method: Option<IndexMethod>,
    options: Vec<IndexOption>,
    predicate: Option<Expr>,
}

impl CreateIndex {
    /// Create a new CREATE INDEX statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_create_index(&mut w, self);
        w
    }

    /// Convert the CREATE INDEX statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_create_index(&mut sql, self);
        sql
    }

    /// Specify the table to create the index on.
    pub fn table<T>(mut self, table: T) -> Self
    where
        T: Into<TableRef>,
    {
        self.table = Some(table.into());
        self
    }

    /// Specify the index name.
    pub fn name<N>(mut self, name: N) -> Self
    where
        N: IntoIden,
    {
        self.name = Some(name.into_iden());
        self
    }

    /// Add a column to the index.
    pub fn column<T>(mut self, column: T) -> Self
    where
        T: IntoIden,
    {
        self.columns.push(Expr::column(column.into_iden()));
        self
    }

    /// Add an expression to the index.
    pub fn expr<E>(mut self, expr: E) -> Self
    where
        E: Into<Expr>,
    {
        self.columns.push(expr.into());
        self
    }

    /// Add a column to the INCLUDE list (covering index).
    pub fn include_column<T>(mut self, column: T) -> Self
    where
        T: IntoIden,
    {
        self.include_columns.push(column.into_iden());
        self
    }

    /// Add columns to the INCLUDE list (covering index).
    pub fn include_columns<I, C>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoIden,
    {
        self.include_columns
            .extend(columns.into_iter().map(IntoIden::into_iden));
        self
    }

    /// Set index as primary
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Set index as unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Create the index if it does not exist.
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Create the index concurrently.
    pub fn concurrently(mut self) -> Self {
        self.concurrently = true;
        self
    }

    /// Specify the index method to use.
    pub fn using<M>(mut self, method: M) -> Self
    where
        M: Into<IndexMethod>,
    {
        self.method = Some(method.into());
        self
    }

    /// Use the GIST index method.
    pub fn gist(mut self) -> Self {
        self.method = Some(IndexMethod::Gist);
        self
    }

    /// Use the BRIN index method.
    pub fn brin(mut self) -> Self {
        self.method = Some(IndexMethod::Brin);
        self
    }

    /// Use the HASH index method.
    pub fn hash(mut self) -> Self {
        self.method = Some(IndexMethod::Hash);
        self
    }

    /// Add a storage parameter to the WITH clause.
    pub fn with_option<N, V>(mut self, name: N, value: V) -> Self
    where
        N: IntoIden,
        V: Into<Expr>,
    {
        self.options.push(IndexOption::new(name, value));
        self
    }

    /// Add storage parameters to the WITH clause.
    pub fn with_options<I, O>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = O>,
        O: Into<IndexOption>,
    {
        self.options.extend(options.into_iter().map(Into::into));
        self
    }

    /// Set the predicate for a partial index.
    pub fn index_where<E>(mut self, expr: E) -> Self
    where
        E: Into<Expr>,
    {
        self.predicate = Some(expr.into());
        self
    }
}

/// Index access method for CREATE INDEX.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IndexMethod {
    /// B-tree index method.
    Btree,
    /// Hash index method.
    Hash,
    /// GIST index method.
    Gist,
    /// BRIN index method.
    Brin,
    /// Custom index method.
    Custom(Cow<'static, str>),
}

impl IndexMethod {
    /// Create a custom index method.
    pub fn custom<T>(method: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self::Custom(method.into())
    }
}

impl From<&'static str> for IndexMethod {
    fn from(value: &'static str) -> Self {
        IndexMethod::Custom(Cow::Borrowed(value))
    }
}

impl From<String> for IndexMethod {
    fn from(value: String) -> Self {
        IndexMethod::Custom(Cow::Owned(value))
    }
}

/// Storage parameter entry for CREATE INDEX.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexOption {
    name: Iden,
    value: Expr,
}

impl IndexOption {
    /// Create a new storage parameter entry.
    pub fn new<N, V>(name: N, value: V) -> Self
    where
        N: IntoIden,
        V: Into<Expr>,
    {
        Self {
            name: name.into_iden(),
            value: value.into(),
        }
    }
}

impl<N, V> From<(N, V)> for IndexOption
where
    N: IntoIden,
    V: Into<Expr>,
{
    fn from((name, value): (N, V)) -> Self {
        Self::new(name, value)
    }
}

fn write_create_index<W: SqlWriter>(w: &mut W, index: &CreateIndex) {
    w.push_str("CREATE INDEX ");
    if index.primary {
        w.push_str("PRIMARY KEY ");
    }
    if index.unique {
        w.push_str("UNIQUE ");
    }
    if index.concurrently {
        w.push_str("CONCURRENTLY ");
    }
    if index.if_not_exists {
        w.push_str("IF NOT EXISTS ");
    }
    if let Some(name) = &index.name {
        write_iden(w, name);
        w.push_char(' ');
    }
    w.push_str("ON ");
    if let Some(table) = &index.table {
        write_table_ref(w, table);
    }
    if let Some(method) = &index.method {
        w.push_str(" USING ");
        write_index_method(w, method);
    }
    if index.table.is_some() || index.method.is_some() {
        w.push_char(' ');
    }
    write_index_columns(w, &index.columns);
    write_index_include(w, &index.include_columns);
    write_index_options(w, &index.options);
    write_index_predicate(w, &index.predicate);
}

/// Write table index definition inside CREATE TABLE statement.
pub(crate) fn write_table_index<W: SqlWriter>(w: &mut W, index: &CreateIndex) {
    if index.primary {
        w.push_str("PRIMARY KEY ");
    }
    if index.unique {
        w.push_str("UNIQUE ");
    }
    write_index_columns(w, &index.columns);
    write_index_include(w, &index.include_columns);
    write_index_options(w, &index.options);
}

fn write_index_columns<W: SqlWriter>(w: &mut W, columns: &[Expr]) {
    w.push_str("(");
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        match col {
            // Wrap opclass expressions in parentheses for disambiguation
            Expr::Binary(_, _, _) | Expr::Unary(_, _) => {
                write_tuple(w, std::slice::from_ref(col));
            }
            _ => {
                write_expr(w, col);
            }
        }
    }
    w.push_str(")");
}

fn write_index_include<W: SqlWriter>(w: &mut W, columns: &[Iden]) {
    if columns.is_empty() {
        return;
    }

    w.push_str(" INCLUDE (");
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_iden(w, col);
    }
    w.push_str(")");
}

fn write_index_method<W: SqlWriter>(w: &mut W, method: &IndexMethod) {
    match method {
        IndexMethod::Btree => w.push_str("btree"),
        IndexMethod::Hash => w.push_str("hash"),
        IndexMethod::Gist => w.push_str("gist"),
        IndexMethod::Brin => w.push_str("brin"),
        IndexMethod::Custom(name) => w.push_str(name),
    }
}

fn write_index_options<W: SqlWriter>(w: &mut W, options: &[IndexOption]) {
    if options.is_empty() {
        return;
    }

    w.push_str(" WITH (");
    for (i, option) in options.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_iden(w, &option.name);
        w.push_str(" = ");
        write_expr(w, &option.value);
    }
    w.push_str(")");
}

fn write_index_predicate<W: SqlWriter>(w: &mut W, predicate: &Option<Expr>) {
    if let Some(expr) = predicate {
        w.push_str(" WHERE ");
        write_expr(w, expr);
    }
}
