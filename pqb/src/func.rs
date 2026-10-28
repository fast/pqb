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

use std::borrow::Cow;

use crate::expr::BinaryOp;
use crate::expr::Expr;
use crate::expr::write_expr;
use crate::types::IntoColumnRef;
use crate::types::IntoIden;
use crate::writer::SqlWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Func {
    Max,
    Min,
    Sum,
    Avg,
    Cast,
    Count,
    Coalesce,
    Lower,
    Upper,
    Custom(Cow<'static, str>),
}

/// A function call expression.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    func: Func,
    args: Vec<Expr>,
    distinct: bool,
    orders: Vec<crate::query::Order>,
    filter: Option<Box<Expr>>,
    window: Option<crate::query::Window>,
}

impl FunctionCall {
    /// Deduplicate aggregate arguments, as in COUNT(DISTINCT column).
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Order aggregate input, as in ARRAY_AGG(value ORDER BY timestamp).
    pub fn order_by(mut self, orders: impl IntoIterator<Item = crate::query::Order>) -> Self {
        self.orders.extend(orders);
        self
    }

    /// Include only rows satisfying this aggregate FILTER condition.
    pub fn filter(mut self, condition: impl Into<Expr>) -> Self {
        self.filter = Some(Box::new(condition.into()));
        self
    }

    /// Evaluate this function over a window.
    pub fn over(mut self, window: crate::query::Window) -> Self {
        self.window = Some(window);
        self
    }

    /// Create ROW_NUMBER() for use with OVER.
    pub fn row_number() -> Self {
        Self::custom("ROW_NUMBER", std::iter::empty::<Expr>())
    }

    /// Create a new MAX function call.
    pub fn max<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Max,
            args: vec![expr.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create a new MIN function call.
    pub fn min<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Min,
            args: vec![expr.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create a new SUM function call.
    pub fn sum<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Sum,
            args: vec![expr.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create a new AVG function call.
    pub fn avg<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Avg,
            args: vec![expr.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create a new COUNT function call.
    pub fn count<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self {
            func: Func::Count,
            args: vec![expr.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create a COUNT(*) function call.
    pub fn count_all() -> Self {
        Self {
            func: Func::Count,
            args: vec![Expr::Asterisk],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create a COALESCE function call.
    pub fn coalesce<A, B>(a: A, b: B) -> Self
    where
        A: Into<Expr>,
        B: Into<Expr>,
    {
        Self {
            func: Func::Coalesce,
            args: vec![a.into(), b.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create an LOWER function call.
    pub fn lower<T>(expr: T) -> FunctionCall
    where
        T: Into<Expr>,
    {
        FunctionCall {
            func: Func::Lower,
            args: vec![expr.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create an UPPER function call.
    pub fn upper<T>(expr: T) -> FunctionCall
    where
        T: Into<Expr>,
    {
        FunctionCall {
            func: Func::Upper,
            args: vec![expr.into()],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Call `CAST` function with a custom type.
    pub fn cast_as<V, I>(expr: V, iden: I) -> FunctionCall
    where
        V: Into<Expr>,
        I: IntoIden,
    {
        let expr = expr.into();
        FunctionCall {
            func: Func::Cast,
            args: vec![expr.binary(BinaryOp::As, Expr::custom(iden.into_iden().into_inner()))],
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }

    /// Create a function call with custom name and arguments.
    pub fn custom<N, T, I>(name: N, args: I) -> Self
    where
        N: Into<Cow<'static, str>>,
        I: IntoIterator<Item = T>,
        T: Into<Expr>,
    {
        Self {
            func: Func::Custom(name.into()),
            args: args.into_iter().map(|arg| arg.into()).collect(),
            distinct: false,
            orders: vec![],
            filter: None,
            window: None,
        }
    }
}

impl From<FunctionCall> for Expr {
    fn from(call: FunctionCall) -> Self {
        Expr::FunctionCall(call)
    }
}

pub(crate) fn write_function_call<W: SqlWriter>(w: &mut W, call: &FunctionCall) {
    match &call.func {
        Func::Max => w.push_str("MAX"),
        Func::Min => w.push_str("MIN"),
        Func::Sum => w.push_str("SUM"),
        Func::Avg => w.push_str("AVG"),
        Func::Cast => w.push_str("CAST"),
        Func::Count => w.push_str("COUNT"),
        Func::Coalesce => w.push_str("COALESCE"),
        Func::Lower => w.push_str("LOWER"),
        Func::Upper => w.push_str("UPPER"),
        Func::Custom(name) => w.push_str(name),
    }
    w.push_char('(');
    if call.distinct {
        w.push_str("DISTINCT ");
    }
    for (i, arg) in call.args.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_expr(w, arg);
    }
    if !call.orders.is_empty() {
        w.push_str(" ORDER BY ");
        for (i, order) in call.orders.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            crate::query::write_order(w, order);
        }
    }
    w.push_char(')');
    if let Some(filter) = &call.filter {
        w.push_str(" FILTER (WHERE ");
        write_expr(w, filter);
        w.push_char(')');
    }
    if let Some(window) = &call.window {
        crate::query::write_window(w, window);
    }
}

/// Express a column reference for use in aggregate functions.
pub fn col<T>(col: T) -> Expr
where
    T: IntoColumnRef,
{
    Expr::column(col)
}
