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

mod common;

use common::ValidateSql;
use pqb::query::Select;
use pqb::value::ArrayError;
use pqb::value::ArrayType;
use pqb::value::ArrayValue;
use pqb::value::Value;

#[test]
fn arrays_validate_shape_bounds_and_element_types() {
    assert!(matches!(
        ArrayValue::new(ArrayType::Int, [2, 2], [1, 2, 3]),
        Err(ArrayError::ElementCount {
            expected: 4,
            actual: 3
        })
    ));
    assert!(matches!(
        ArrayValue::new(ArrayType::Int, [1; 7], [1]),
        Err(ArrayError::TooManyDimensions)
    ));
    assert!(matches!(
        ArrayValue::new(ArrayType::Int, [(1, i32::MAX)], [1]),
        Err(ArrayError::DimensionOverflow)
    ));
    assert!(matches!(
        ArrayValue::new(ArrayType::Int, [usize::MAX], [1]),
        Err(ArrayError::DimensionOverflow)
    ));
    assert!(matches!(
        ArrayValue::new(ArrayType::Int, [65536, 65536], [1]),
        Err(ArrayError::DimensionOverflow)
    ));
    assert!(matches!(
        ArrayValue::new(
            ArrayType::Int,
            [2],
            [Value::from(1), Value::from(None::<String>)]
        ),
        Err(ArrayError::ElementType { index: 1, .. })
    ));
    let empty = ArrayValue::new(ArrayType::Int, [2, 0, 3], Vec::<i32>::new()).unwrap();
    assert!(empty.dimensions().is_empty());
    assert!(empty.values().is_empty());
}

#[test]
fn multidimensional_arrays_remain_a_single_parameter() {
    let matrix = ArrayValue::new(
        ArrayType::Int,
        [(2, 0), (2, -1)],
        [Some(1), None, Some(3), Some(4)],
    )
    .unwrap();
    let query = Select::new().expr(matrix.clone()).expr("after");
    query.to_sql().validate();
    let bound = query.to_values();
    assert_eq!(bound.sql(), "SELECT $1, $2");
    assert_eq!(bound.values(), &[matrix.into(), "after".into()]);
}
