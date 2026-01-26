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

//! SQL built-in functions.

use crate::expr::Expr;
use crate::expr::write_expr;
use crate::types::IntoColumnRef;
use crate::writer::SqlWriter;

/// SQL built-in functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Func {
    /// MAX function.
    Max,
    /// MIN function.
    Min,
    /// SUM function.
    Sum,
    /// AVG function.
    Avg,
    /// COUNT function.
    Count,
}

/// A function call expression.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub(crate) func: Func,
    pub(crate) expr: Box<Expr>,
}

impl FunctionCall {
    /// Create a new MAX function call.
    pub fn max<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Max,
            expr: Box::new(expr.into()),
        }
    }

    /// Create a new MIN function call.
    pub fn min<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Min,
            expr: Box::new(expr.into()),
        }
    }

    /// Create a new SUM function call.
    pub fn sum<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Sum,
            expr: Box::new(expr.into()),
        }
    }

    /// Create a new AVG function call.
    pub fn avg<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Avg,
            expr: Box::new(expr.into()),
        }
    }

    /// Create a new COUNT function call.
    pub fn count<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Count,
            expr: Box::new(expr.into()),
        }
    }

    /// Create a COUNT(*) function call.
    pub fn count_all() -> Self {
        Self {
            func: Func::Count,
            expr: Box::new(Expr::Asterisk),
        }
    }
}

impl From<FunctionCall> for Expr {
    fn from(call: FunctionCall) -> Self {
        Expr::FunctionCall(call)
    }
}

pub(crate) fn write_function_call<W: SqlWriter>(w: &mut W, call: &FunctionCall) {
    match call.func {
        Func::Max => w.push_str("MAX"),
        Func::Min => w.push_str("MIN"),
        Func::Sum => w.push_str("SUM"),
        Func::Avg => w.push_str("AVG"),
        Func::Count => w.push_str("COUNT"),
    }
    w.push_char('(');
    write_expr(w, &call.expr);
    w.push_char(')');
}

/// Express a column reference for use in aggregate functions.
pub fn col<T>(col: T) -> Expr
where
    T: IntoColumnRef,
{
    Expr::column(col)
}
