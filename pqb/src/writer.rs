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

use std::fmt::Write;

use crate::value::Value;
use crate::value::write_string_value;

pub trait SqlWriter {
    fn push_param(&mut self, value: Value);

    fn push_str(&mut self, value: &str);

    fn push_char(&mut self, value: char);
}

impl SqlWriter for String {
    fn push_param(&mut self, value: Value) {
        match value {
            Value::Bool(None)
            | Value::TinyInt(None)
            | Value::SmallInt(None)
            | Value::Int(None)
            | Value::BigInt(None)
            | Value::TinyUnsigned(None)
            | Value::SmallUnsigned(None)
            | Value::Unsigned(None)
            | Value::BigUnsigned(None)
            | Value::Float(None)
            | Value::Double(None)
            | Value::String(None) => self.push_str("NULL"),

            Value::Bool(Some(b)) => self.push_str(if b { "TRUE" } else { "FALSE" }),
            Value::TinyInt(Some(i)) => write!(self, "{i}").unwrap(),
            Value::SmallInt(Some(i)) => write!(self, "{i}").unwrap(),
            Value::Int(Some(i)) => write!(self, "{i}").unwrap(),
            Value::BigInt(Some(i)) => write!(self, "{i}").unwrap(),
            Value::TinyUnsigned(Some(u)) => write!(self, "{u}").unwrap(),
            Value::SmallUnsigned(Some(u)) => write!(self, "{u}").unwrap(),
            Value::Unsigned(Some(u)) => write!(self, "{u}").unwrap(),
            Value::BigUnsigned(Some(u)) => write!(self, "{u}").unwrap(),
            Value::Float(Some(f)) => write!(self, "{f}").unwrap(),
            Value::Double(Some(f)) => write!(self, "{f}").unwrap(),
            Value::String(Some(s)) => write_string_value(self, s.as_str()),
        }
    }

    fn push_str(&mut self, value: &str) {
        String::push_str(self, value)
    }

    fn push_char(&mut self, value: char) {
        String::push(self, value)
    }
}
