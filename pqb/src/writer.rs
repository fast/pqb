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

use std::fmt::Arguments;
use std::fmt::Write;

use crate::value::Value;
use crate::value::write_value;

pub trait SqlWriter {
    fn push_param(&mut self, value: Value);

    fn push_str(&mut self, value: &str);

    fn push_char(&mut self, value: char);

    fn push_fmt(&mut self, args: Arguments);
}

impl SqlWriter for String {
    fn push_param(&mut self, value: Value) {
        write_value(self, &value);
    }

    fn push_str(&mut self, value: &str) {
        String::push_str(self, value)
    }

    fn push_char(&mut self, value: char) {
        String::push(self, value)
    }

    fn push_fmt(&mut self, args: Arguments) {
        self.write_fmt(args).unwrap();
    }
}

/// SQL writer that collects parameters for prepared statements.
pub struct SqlWriterValues {
    sql: String,
    values: Vec<Value>,
    counter: usize,
}

impl SqlWriterValues {
    /// Create a new writer for PostgreSQL placeholder style ($1, $2, ...).
    pub fn new() -> Self {
        Self {
            sql: String::new(),
            values: Vec::new(),
            counter: 0,
        }
    }

    /// Consume the writer and return the SQL string and collected values.
    pub fn into_parts(self) -> (String, Vec<Value>) {
        (self.sql, self.values)
    }
}

impl Default for SqlWriterValues {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlWriter for SqlWriterValues {
    fn push_param(&mut self, value: Value) {
        self.counter += 1;
        write!(self.sql, "${}", self.counter).unwrap();
        self.values.push(value);
    }

    fn push_str(&mut self, value: &str) {
        self.sql.push_str(value);
    }

    fn push_char(&mut self, value: char) {
        self.sql.push(value);
    }

    fn push_fmt(&mut self, args: Arguments) {
        self.sql.write_fmt(args).unwrap();
    }
}
