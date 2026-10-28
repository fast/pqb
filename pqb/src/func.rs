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
    RowNumber,
    Coalesce,
    Lower,
    Upper,
    Custom(Cow<'static, str>),
}

/// A function call expression.
///
/// Rendering rejects unsupported modifiers on built-in scalar functions, aggregate wildcards,
/// and windows. PostgreSQL checks the signatures and aggregate status of custom functions.
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
    fn new(func: Func, args: Vec<Expr>) -> Self {
        Self {
            func,
            args,
            distinct: false,
            orders: Vec::new(),
            filter: None,
            window: None,
        }
    }

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
        Self::new(Func::RowNumber, Vec::new())
    }

    /// Create a new MAX function call.
    pub fn max<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self::new(Func::Max, vec![expr.into()])
    }

    /// Create a new MIN function call.
    pub fn min<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self::new(Func::Min, vec![expr.into()])
    }

    /// Create a new SUM function call.
    pub fn sum<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self::new(Func::Sum, vec![expr.into()])
    }

    /// Create a new AVG function call.
    pub fn avg<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self::new(Func::Avg, vec![expr.into()])
    }

    /// Create a new COUNT function call.
    pub fn count<T>(expr: T) -> Self
    where
        T: Into<Expr>,
    {
        Self::new(Func::Count, vec![expr.into()])
    }

    /// Create a COUNT(*) function call.
    pub fn count_all() -> Self {
        Self::new(Func::Count, vec![Expr::Asterisk])
    }

    /// Create a COALESCE function call.
    pub fn coalesce<A, B>(a: A, b: B) -> Self
    where
        A: Into<Expr>,
        B: Into<Expr>,
    {
        Self::new(Func::Coalesce, vec![a.into(), b.into()])
    }

    /// Create an LOWER function call.
    pub fn lower<T>(expr: T) -> FunctionCall
    where
        T: Into<Expr>,
    {
        Self::new(Func::Lower, vec![expr.into()])
    }

    /// Create an UPPER function call.
    pub fn upper<T>(expr: T) -> FunctionCall
    where
        T: Into<Expr>,
    {
        Self::new(Func::Upper, vec![expr.into()])
    }

    /// Call `CAST` function with a custom type.
    pub fn cast_as<V, I>(expr: V, iden: I) -> FunctionCall
    where
        V: Into<Expr>,
        I: IntoIden,
    {
        let expr = expr.into();
        Self::new(
            Func::Cast,
            vec![expr.binary(BinaryOp::As, Expr::custom(iden.into_iden().into_inner()))],
        )
    }

    /// Create a function call with custom name and arguments.
    pub fn custom<N, T, I>(name: N, args: I) -> Self
    where
        N: Into<Cow<'static, str>>,
        I: IntoIterator<Item = T>,
        T: Into<Expr>,
    {
        Self::new(
            Func::Custom(name.into()),
            args.into_iter().map(|arg| arg.into()).collect(),
        )
    }
}

impl From<FunctionCall> for Expr {
    fn from(call: FunctionCall) -> Self {
        Expr::FunctionCall(call)
    }
}

pub(crate) fn write_function_call<W: SqlWriter>(w: &mut W, call: &FunctionCall) {
    if matches!(
        call.func,
        Func::Cast | Func::Coalesce | Func::Lower | Func::Upper
    ) {
        assert!(
            !call.distinct
                && call.orders.is_empty()
                && call.filter.is_none()
                && call.window.is_none(),
            "scalar functions do not accept aggregate or window modifiers"
        );
    }
    if call.func == Func::RowNumber {
        assert!(call.window.is_some(), "ROW_NUMBER requires OVER");
        assert!(
            !call.distinct && call.orders.is_empty() && call.filter.is_none(),
            "ROW_NUMBER does not accept aggregate modifiers"
        );
    }
    if call.window.is_some() {
        assert!(
            !call.distinct && call.orders.is_empty(),
            "PostgreSQL window functions do not support DISTINCT or aggregate ORDER BY"
        );
    }
    let star = call.args.iter().any(|arg| {
        matches!(
            arg,
            Expr::Asterisk | Expr::Column(crate::types::ColumnRef::Asterisk(None))
        )
    });
    if star {
        assert!(
            matches!(call.func, Func::Count | Func::Custom(_))
                && call.args.len() == 1
                && !call.distinct
                && call.orders.is_empty(),
            "invalid aggregate wildcard argument"
        );
    }
    assert!(
        !call.args.is_empty() || (!call.distinct && call.orders.is_empty()),
        "aggregate DISTINCT and ORDER BY require arguments"
    );

    match &call.func {
        Func::Max => w.push_str("MAX"),
        Func::Min => w.push_str("MIN"),
        Func::Sum => w.push_str("SUM"),
        Func::Avg => w.push_str("AVG"),
        Func::Cast => w.push_str("CAST"),
        Func::Count => w.push_str("COUNT"),
        Func::RowNumber => w.push_str("ROW_NUMBER"),
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
