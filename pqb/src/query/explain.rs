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
use crate::query::Delete;
use crate::query::Insert;
use crate::query::Select;
use crate::query::Update;
use crate::query::write_delete;
use crate::query::write_insert;
use crate::query::write_select;
use crate::query::write_update;
use crate::writer::SqlWriter;

/// Explain a SQL statement.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Explain {
    statement: Option<ExplainableStatement>,
    analyze: Option<bool>,
    format: Option<Format>,
    verbose: Option<bool>,
    costs: Option<bool>,
    settings: Option<bool>,
    generic_plan: Option<bool>,
    buffers: Option<bool>,
    serialize: Option<Serialize>,
    wal: Option<bool>,
    timing: Option<bool>,
    summary: Option<bool>,
    memory: Option<bool>,
}

impl Explain {
    /// Create a new EXPLAIN statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_explain(&mut w, self);
        w
    }

    /// Convert the EXPLAIN statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_explain(&mut sql, self);
        sql
    }

    /// Set the statement to be explained.
    pub fn statement(mut self, statement: impl Into<ExplainableStatement>) -> Self {
        self.statement = Some(statement.into());
        self
    }

    /// Set `ANALYZE` to `true`.
    pub fn analyze(mut self) -> Self {
        self.analyze = Some(true);
        self
    }

    /// Set `FORMAT TEXT`.
    pub fn format_text(mut self) -> Self {
        self.format = Some(Format::Text);
        self
    }

    /// Set `FORMAT XML`.
    pub fn format_xml(mut self) -> Self {
        self.format = Some(Format::Xml);
        self
    }

    /// Set `FORMAT JSON`.
    pub fn format_json(mut self) -> Self {
        self.format = Some(Format::Json);
        self
    }

    /// Set `FORMAT YAML`.
    pub fn format_yaml(mut self) -> Self {
        self.format = Some(Format::Yaml);
        self
    }

    /// Set `VERBOSE`.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = Some(verbose);
        self
    }

    /// Set `COSTS`.
    pub fn costs(mut self, costs: bool) -> Self {
        self.costs = Some(costs);
        self
    }

    /// Set `SETTINGS`.
    pub fn settings(mut self, settings: bool) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Set `GENERIC_PLAN`.
    pub fn generic_plan(mut self, generic_plan: bool) -> Self {
        self.generic_plan = Some(generic_plan);
        self
    }

    /// Set `BUFFERS`.
    pub fn buffers(mut self, buffers: bool) -> Self {
        self.buffers = Some(buffers);
        self
    }

    /// Set `SERIALIZE TEXT`.
    pub fn serialize_text(mut self) -> Self {
        self.serialize = Some(Serialize::Text);
        self
    }

    /// Set `SERIALIZE BINARY`.
    pub fn serialize_binary(mut self) -> Self {
        self.serialize = Some(Serialize::Binary);
        self
    }

    /// Set `SERIALIZE NONE`.
    pub fn serialize_none(mut self) -> Self {
        self.serialize = Some(Serialize::None);
        self
    }

    /// Set `WAL`.
    pub fn wal(mut self, wal: bool) -> Self {
        self.wal = Some(wal);
        self
    }

    /// Set `TIMING`.
    pub fn timing(mut self, timing: bool) -> Self {
        self.timing = Some(timing);
        self
    }

    /// Set `SUMMARY`.
    pub fn summary(mut self, summary: bool) -> Self {
        self.summary = Some(summary);
        self
    }

    /// Set `MEMORY`.
    pub fn memory(mut self, memory: bool) -> Self {
        self.memory = Some(memory);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum ExplainableStatement {
    Select(Select),
    Insert(Insert),
    Update(Update),
    Delete(Delete),
}

impl From<Select> for ExplainableStatement {
    fn from(s: Select) -> Self {
        ExplainableStatement::Select(s)
    }
}

impl From<Insert> for ExplainableStatement {
    fn from(i: Insert) -> Self {
        ExplainableStatement::Insert(i)
    }
}

impl From<Update> for ExplainableStatement {
    fn from(u: Update) -> Self {
        ExplainableStatement::Update(u)
    }
}

impl From<Delete> for ExplainableStatement {
    fn from(d: Delete) -> Self {
        ExplainableStatement::Delete(d)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Text,
    Xml,
    Json,
    Yaml,
}

impl Format {
    const fn as_str(&self) -> &'static str {
        match self {
            Format::Text => "TEXT",
            Format::Xml => "XML",
            Format::Json => "JSON",
            Format::Yaml => "YAML",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Serialize {
    None,
    Text,
    Binary,
}

impl Serialize {
    const fn as_str(&self) -> &'static str {
        match self {
            Serialize::None => "NONE",
            Serialize::Text => "TEXT",
            Serialize::Binary => "BINARY",
        }
    }
}

fn write_explain<W: SqlWriter>(w: &mut W, explain: &Explain) {
    w.push_str("EXPLAIN");

    let has_options = explain.analyze.is_some()
        || explain.verbose.is_some()
        || explain.costs.is_some()
        || explain.settings.is_some()
        || explain.generic_plan.is_some()
        || explain.buffers.is_some()
        || explain.serialize.is_some()
        || explain.wal.is_some()
        || explain.timing.is_some()
        || explain.summary.is_some()
        || explain.memory.is_some()
        || explain.format.is_some();

    if has_options {
        w.push_str(" (");

        // Specifies whether the selected option should be turned on or off. You can write TRUE, ON,
        // or 1 to enable the option, and FALSE, OFF, or 0 to disable it. The boolean value can also
        // be omitted, in which case TRUE is assumed.
        //
        // @see https://www.postgresql.org/docs/18/sql-explain.html
        fn write_false<W: SqlWriter>(w: &mut W, value: bool) {
            if !value {
                w.push_str(" 0");
            }
        }

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
        if let Some(analyze) = explain.analyze {
            write_comma_if_not_first!();
            w.push_str("ANALYZE");
            write_false(w, analyze);
        }
        if let Some(verbose) = explain.verbose {
            write_comma_if_not_first!();
            w.push_str("VERBOSE");
            write_false(w, verbose);
        }
        if let Some(costs) = explain.costs {
            write_comma_if_not_first!();
            w.push_str("COSTS");
            write_false(w, costs);
        }
        if let Some(settings) = explain.settings {
            write_comma_if_not_first!();
            w.push_str("SETTINGS");
            write_false(w, settings);
        }
        if let Some(generic_plan) = explain.generic_plan {
            write_comma_if_not_first!();
            w.push_str("GENERIC_PLAN");
            write_false(w, generic_plan);
        }
        if let Some(buffers) = explain.buffers {
            write_comma_if_not_first!();
            w.push_str("BUFFERS");
            write_false(w, buffers);
        }
        if let Some(serialize) = explain.serialize {
            write_comma_if_not_first!();
            w.push_str("SERIALIZE ");
            w.push_str(serialize.as_str());
        }
        if let Some(wal) = explain.wal {
            write_comma_if_not_first!();
            w.push_str("WAL");
            write_false(w, wal);
        }
        if let Some(timing) = explain.timing {
            write_comma_if_not_first!();
            w.push_str("TIMING");
            write_false(w, timing);
        }
        if let Some(summary) = explain.summary {
            write_comma_if_not_first!();
            w.push_str("SUMMARY");
            write_false(w, summary);
        }
        if let Some(memory) = explain.memory {
            write_comma_if_not_first!();
            w.push_str("MEMORY");
            write_false(w, memory);
        }
        if let Some(format) = explain.format {
            write_comma_if_not_first!();
            w.push_str("FORMAT ");
            w.push_str(format.as_str());
        }
        let _ = is_first;

        w.push_str(")");
    }

    if let Some(statement) = &explain.statement {
        w.push_str(" ");
        match statement {
            ExplainableStatement::Select(s) => write_select(w, s),
            ExplainableStatement::Insert(i) => write_insert(w, i),
            ExplainableStatement::Update(u) => write_update(w, u),
            ExplainableStatement::Delete(d) => write_delete(w, d),
        }
    }
}
