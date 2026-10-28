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
use crate::query::Statement;
use crate::query::write_statement;
use crate::types::Iden;
use crate::types::IntoIden;
use crate::types::TableName;
use crate::types::write_iden;
use crate::types::write_table_name;
use crate::value::Value;
use crate::value::write_value;
use crate::writer::SqlWriter;

/// The representation of COPY data. Encoding and streaming the data belong to the driver.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum CopyFormat {
    #[default]
    Text,
    Csv,
    Binary,
}

/// COPY a table from STDIN, or a table/query to STDOUT.
///
/// Files and server programs can be selected explicitly. COPY is a utility statement: query
/// expressions, options and WHERE values are escaped inline, including in `to_values()`.
/// Use your driver's COPY API to transfer data; these builders do not encode rows.
#[derive(Debug, Clone, PartialEq)]
pub struct Copy {
    relation: CopyRelation,
    from: bool,
    location: Location,
    columns: Vec<Iden>,
    format: CopyFormat,
    delimiter: Option<char>,
    null: Option<String>,
    header: Option<bool>,
    quote: Option<char>,
    escape: Option<char>,
    encoding: Option<String>,
    freeze: bool,
    force_quote: Option<Vec<Iden>>,
    force_null: Vec<Iden>,
    force_not_null: Vec<Iden>,
    conditions: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
enum CopyRelation {
    Table(TableName),
    Query(Box<Statement>),
}

#[derive(Debug, Clone, PartialEq)]
enum Location {
    Stream,
    File(String),
    Program(String),
}

impl Copy {
    /// Copy data from STDIN into a table.
    pub fn from_table(table: impl Into<TableName>) -> Self {
        Self::new(CopyRelation::Table(table.into()), true)
    }

    /// Copy a table to STDOUT. Use `from_query` to include partitions or inherited tables.
    pub fn to_table(table: impl Into<TableName>) -> Self {
        Self::new(CopyRelation::Table(table.into()), false)
    }

    /// Copy a SELECT or a data-modifying query with RETURNING to STDOUT.
    pub fn from_query(query: impl Into<Statement>) -> Self {
        Self::new(CopyRelation::Query(Box::new(query.into())), false)
    }

    fn new(relation: CopyRelation, from: bool) -> Self {
        Self {
            relation,
            from,
            location: Location::Stream,
            columns: Vec::new(),
            format: CopyFormat::Text,
            delimiter: None,
            null: None,
            header: None,
            quote: None,
            escape: None,
            encoding: None,
            freeze: false,
            force_quote: None,
            force_null: Vec::new(),
            force_not_null: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Read or write a file on the database server instead of the client stream.
    /// PostgreSQL checks the path and required server-side privileges.
    pub fn file(mut self, path: impl Into<String>) -> Self {
        self.location = Location::File(path.into());
        self
    }

    /// Read or write a program's stream on the database server.
    ///
    /// The command is SQL-escaped, but PostgreSQL executes it through a shell. Supply only a
    /// trusted command, never interpolated user input. Server-side privileges are required.
    pub fn program(mut self, command: impl Into<String>) -> Self {
        self.location = Location::Program(command.into());
        self
    }

    /// Select table columns in data order. Not applicable to a query source.
    pub fn columns<C: IntoIden>(mut self, columns: impl IntoIterator<Item = C>) -> Self {
        assert!(
            matches!(self.relation, CopyRelation::Table(_)),
            "COPY query columns belong in the query projection"
        );
        self.columns
            .extend(columns.into_iter().map(IntoIden::into_iden));
        self
    }

    /// Set text, CSV or binary format (default: text).
    pub fn format(mut self, format: CopyFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the single-byte text/CSV field separator.
    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = Some(delimiter);
        self
    }

    /// Set the text/CSV marker for a NULL field.
    pub fn null(mut self, marker: impl Into<String>) -> Self {
        self.null = Some(marker.into());
        self
    }

    /// Read or write a header line. Text-format headers require PostgreSQL 15+.
    pub fn header(mut self, header: bool) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the single-byte CSV quotation character.
    pub fn quote(mut self, quote: char) -> Self {
        self.quote = Some(quote);
        self
    }

    /// Set the single-byte CSV escape character.
    pub fn escape(mut self, escape: char) -> Self {
        self.escape = Some(escape);
        self
    }

    /// Set the data encoding, using a PostgreSQL encoding name.
    pub fn encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = Some(encoding.into());
        self
    }

    /// Freeze imported rows. PostgreSQL enforces transaction and table eligibility.
    pub fn freeze(mut self) -> Self {
        assert!(self.from, "FREEZE requires COPY FROM");
        self.freeze = true;
        self
    }

    /// Quote every non-NULL field in CSV output.
    pub fn force_quote_all(mut self) -> Self {
        assert!(!self.from, "FORCE_QUOTE requires COPY TO");
        self.force_quote = Some(Vec::new());
        self
    }

    /// Quote non-NULL fields in these CSV output columns.
    pub fn force_quote<C: IntoIden>(mut self, columns: impl IntoIterator<Item = C>) -> Self {
        assert!(!self.from, "FORCE_QUOTE requires COPY TO");
        let columns: Vec<_> = columns.into_iter().map(IntoIden::into_iden).collect();
        assert!(
            !columns.is_empty(),
            "FORCE_QUOTE requires columns; use force_quote_all for all columns"
        );
        self.force_quote = Some(columns);
        self
    }

    /// Treat quoted NULL markers as NULL in these CSV input columns.
    pub fn force_null<C: IntoIden>(mut self, columns: impl IntoIterator<Item = C>) -> Self {
        assert!(self.from, "FORCE_NULL requires COPY FROM");
        self.force_null
            .extend(columns.into_iter().map(IntoIden::into_iden));
        self
    }

    /// Preserve unquoted NULL markers as data in these CSV input columns.
    pub fn force_not_null<C: IntoIden>(mut self, columns: impl IntoIterator<Item = C>) -> Self {
        assert!(self.from, "FORCE_NOT_NULL requires COPY FROM");
        self.force_not_null
            .extend(columns.into_iter().map(IntoIden::into_iden));
        self
    }

    /// Filter input rows during COPY FROM. For output filtering, use `from_query`.
    pub fn and_where(mut self, condition: impl Into<Expr>) -> Self {
        assert!(self.from, "COPY TO filters belong in the source query");
        self.conditions.push(condition.into());
        self
    }

    /// Render a COPY command with escaped inline values.
    pub fn to_sql(&self) -> String {
        self.validate();
        let mut sql = String::from("COPY ");
        match &self.relation {
            CopyRelation::Table(table) => {
                write_table_name(&mut sql, table);
                if !self.columns.is_empty() {
                    sql.push(' ');
                    write_columns(&mut sql, &self.columns);
                }
            }
            CopyRelation::Query(query) => {
                sql.push('(');
                write_statement(&mut sql, query);
                sql.push(')');
            }
        }
        sql.push_str(if self.from { " FROM " } else { " TO " });
        match &self.location {
            Location::Stream => sql.push_str(if self.from { "STDIN" } else { "STDOUT" }),
            Location::File(path) => write_value(&mut sql, &Value::from(path.clone())),
            Location::Program(command) => {
                sql.push_str("PROGRAM ");
                write_value(&mut sql, &Value::from(command.clone()));
            }
        }
        sql.push_str(" WITH (FORMAT ");
        sql.push_str(match self.format {
            CopyFormat::Text => "TEXT",
            CopyFormat::Csv => "CSV",
            CopyFormat::Binary => "BINARY",
        });
        for (name, value) in [
            ("DELIMITER", self.delimiter),
            ("QUOTE", self.quote),
            ("ESCAPE", self.escape),
        ] {
            if let Some(value) = value {
                sql.push_fmt(format_args!(", {name} "));
                write_value(&mut sql, &Value::from(value.to_string()));
            }
        }
        for (name, value) in [("NULL", &self.null), ("ENCODING", &self.encoding)] {
            if let Some(value) = value {
                sql.push_fmt(format_args!(", {name} "));
                write_value(&mut sql, &Value::from(value.clone()));
            }
        }
        if let Some(header) = self.header {
            sql.push_str(if header {
                ", HEADER TRUE"
            } else {
                ", HEADER FALSE"
            });
        }
        if self.freeze {
            sql.push_str(", FREEZE TRUE");
        }
        if let Some(columns) = &self.force_quote {
            sql.push_str(", FORCE_QUOTE ");
            if columns.is_empty() {
                sql.push('*');
            } else {
                write_columns(&mut sql, columns);
            }
        }
        for (name, columns) in [
            ("FORCE_NULL", &self.force_null),
            ("FORCE_NOT_NULL", &self.force_not_null),
        ] {
            if !columns.is_empty() {
                sql.push_fmt(format_args!(", {name} "));
                write_columns(&mut sql, columns);
            }
        }
        sql.push(')');
        if let Some(condition) = Expr::from_conditions(self.conditions.clone()) {
            sql.push_str(" WHERE ");
            write_expr(&mut sql, &condition);
        }
        sql
    }

    /// Return the utility SQL and an empty parameter list. COPY cannot bind parameters.
    pub fn to_values(&self) -> SqlWriterValues {
        SqlWriterValues::from_sql(self.to_sql())
    }

    fn validate(&self) {
        let csv_options = self.quote.is_some()
            || self.escape.is_some()
            || self.force_quote.is_some()
            || !self.force_null.is_empty()
            || !self.force_not_null.is_empty();
        assert!(
            !csv_options || self.format == CopyFormat::Csv,
            "CSV options require FORMAT CSV"
        );
        assert!(
            self.format != CopyFormat::Binary
                || (self.delimiter.is_none() && self.null.is_none() && self.header.is_none()),
            "binary COPY does not support DELIMITER, NULL or HEADER"
        );
        for character in [self.delimiter, self.quote, self.escape]
            .into_iter()
            .flatten()
        {
            assert!(
                character.is_ascii() && !matches!(character, '\0' | '\r' | '\n'),
                "COPY separators must be a single non-NUL byte other than CR or LF"
            );
        }
        let delimiter = self.delimiter.unwrap_or(if self.format == CopyFormat::Csv {
            ','
        } else {
            '\t'
        });
        let quote = self.quote.unwrap_or('"');
        if self.format == CopyFormat::Csv {
            assert_ne!(delimiter, quote, "COPY delimiter and quote must differ");
        }
        if self.format == CopyFormat::Text {
            assert!(
                !matches!(delimiter, '\\' | '.' | 'a'..='z' | '0'..='9'),
                "invalid text COPY delimiter"
            );
        }
        if let Some(null) = &self.null {
            assert!(
                !null.contains(['\0', '\r', '\n', delimiter]),
                "COPY NULL marker contains a separator"
            );
            assert!(
                self.format != CopyFormat::Csv || !null.contains(quote),
                "CSV NULL marker contains the quote character"
            );
        }
    }
}

fn write_columns(sql: &mut String, columns: &[Iden]) {
    sql.push('(');
    for (i, column) in columns.iter().enumerate() {
        if i > 0 {
            sql.push_str(", ");
        }
        write_iden(sql, column);
    }
    sql.push(')');
}
