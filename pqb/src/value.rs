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

//! Container for all SQL value types.

use crate::writer::SqlWriter;

/// SQL value variants.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Boolean value.
    Bool(Option<bool>),
    /// Tiny integer value.
    TinyInt(Option<i8>),
    /// Small integer value.
    SmallInt(Option<i16>),
    /// Integer value.
    Int(Option<i32>),
    /// Big integer value.
    BigInt(Option<i64>),
    /// Tiny unsigned integer value.
    TinyUnsigned(Option<u8>),
    /// Small unsigned integer value.
    SmallUnsigned(Option<u16>),
    /// Unsigned integer value.
    Unsigned(Option<u32>),
    /// Big unsigned integer value.
    BigUnsigned(Option<u64>),
    /// Floating point value.
    Float(Option<f32>),
    /// Double precision floating point value.
    Double(Option<f64>),
}

macro_rules! type_to_value {
    ( $type: ty, $name: ident ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(x))
            }
        }
    };
}

type_to_value!(bool, Bool);
type_to_value!(i8, TinyInt);
type_to_value!(i16, SmallInt);
type_to_value!(i32, Int);
type_to_value!(i64, BigInt);
type_to_value!(u8, TinyUnsigned);
type_to_value!(u16, SmallUnsigned);
type_to_value!(u32, Unsigned);
type_to_value!(u64, BigUnsigned);
type_to_value!(f32, Float);
type_to_value!(f64, Double);

pub(crate) fn write_value<W: SqlWriter>(w: &mut W, value: Value) {
    w.push_param(value);
}
