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

//! Building blocks of SQL statements.
//!
//! [`Expr`] is an arbitrary, dynamically-typed SQL expression. It can be used in select fields,
//! where clauses and many other places.

use std::borrow::Cow;

use crate::func::FunctionCall;
use crate::func::write_function_call;
use crate::query::Select;
use crate::query::write_select;
use crate::types::ColumnName;
use crate::types::ColumnRef;
use crate::types::IntoColumnRef;
use crate::types::write_iden;
use crate::types::write_table_name;
use crate::value::Value;
use crate::value::write_value;
use crate::writer::SqlWriter;

/// SQL keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum Keyword {
    Null,
}

/// An arbitrary, dynamically-typed SQL expression.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum Expr {
    Column(ColumnRef),
    Asterisk,
    Keyword(Keyword),
    Tuple(Vec<Expr>),
    Value(Value),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    FunctionCall(FunctionCall),
    SubQuery(Option<SubQueryOp>, Box<Select>),
    Custom(Cow<'static, str>),
}

/// # Expression constructors
impl Expr {
    /// Express a [`Value`], returning a [`Expr`].
    pub fn value<T>(value: T) -> Expr
    where
        T: Into<Value>,
    {
        Expr::Value(value.into())
    }

    /// Express the target column, returning a [`Expr`].
    pub fn column<T>(n: T) -> Self
    where
        T: IntoColumnRef,
    {
        Expr::Column(n.into_column_ref())
    }

    /// Express the asterisk (*) without table prefix.
    pub fn asterisk() -> Self {
        Expr::Asterisk
    }

    /// Wraps tuple of `Expr`, can be used for tuple comparison.
    pub fn tuple<I>(n: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Expr::Tuple(n.into_iter().collect())
    }

    /// Express a custom SQL expression.
    pub fn custom<T>(expr: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Expr::Custom(expr.into())
    }
}

/// # Expression combinators
impl Expr {
    /// Create a MAX() function call.
    pub fn max(self) -> Self {
        Expr::FunctionCall(FunctionCall::max(self))
    }

    /// Create a MIN() function call.
    pub fn min(self) -> Self {
        Expr::FunctionCall(FunctionCall::min(self))
    }

    /// Create a SUM() function call.
    pub fn sum(self) -> Self {
        Expr::FunctionCall(FunctionCall::sum(self))
    }

    /// Create an AVG() function call.
    pub fn avg(self) -> Self {
        Expr::FunctionCall(FunctionCall::avg(self))
    }

    /// Create a COUNT() function call.
    pub fn count(self) -> Self {
        Expr::FunctionCall(FunctionCall::count(self))
    }

    /// Check if the expression is NULL.
    pub fn is_null(self) -> Self {
        self.binary(BinaryOp::Is, Expr::Keyword(Keyword::Null))
    }

    /// Check if the expression is NOT NULL.
    pub fn is_not_null(self) -> Self {
        self.binary(BinaryOp::IsNot, Expr::Keyword(Keyword::Null))
    }

    /// Check if the expression is between two values.
    pub fn between<A, B>(self, a: A, b: B) -> Self
    where
        A: Into<Expr>,
        B: Into<Expr>,
    {
        self.binary(
            BinaryOp::Between,
            Expr::Binary(Box::new(a.into()), BinaryOp::And, Box::new(b.into())),
        )
    }

    /// Check if the expression is not between two values.
    pub fn not_between<A, B>(self, a: A, b: B) -> Self
    where
        A: Into<Expr>,
        B: Into<Expr>,
    {
        self.binary(
            BinaryOp::NotBetween,
            Expr::Binary(Box::new(a.into()), BinaryOp::And, Box::new(b.into())),
        )
    }

    /// Pattern matching with LIKE.
    pub fn like<R>(self, pattern: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Like, pattern)
    }

    /// Add a value.
    #[expect(clippy::should_implement_trait)]
    pub fn add<R>(self, rhs: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Add, rhs)
    }

    /// Subtract a value.
    #[expect(clippy::should_implement_trait)]
    pub fn sub<R>(self, rhs: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Sub, rhs)
    }

    /// Multiply by a value.
    #[expect(clippy::should_implement_trait)]
    pub fn mul<R>(self, rhs: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Mul, rhs)
    }

    /// Divide by a value.
    #[expect(clippy::should_implement_trait)]
    pub fn div<R>(self, rhs: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Div, rhs)
    }

    /// Replace NULL with the specified value using COALESCE.
    pub fn if_null<V>(self, value: V) -> Self
    where
        V: Into<Expr>,
    {
        Expr::FunctionCall(FunctionCall::coalesce(self, value))
    }

    /// Greater than (`>`).
    pub fn gt<R>(self, right: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::GreaterThan, right)
    }

    /// Greater than or equal (`>=`).
    pub fn gte<R>(self, right: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::GreaterThanOrEqual, right)
    }

    /// Less than (`<`).
    pub fn lt<R>(self, right: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::LessThan, right)
    }

    /// Less than or equal (`<=`).
    pub fn lte<R>(self, right: R) -> Self
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::LessThanOrEqual, right)
    }

    /// Create any binary operation.
    pub fn binary<R>(self, op: BinaryOp, rhs: R) -> Self
    where
        R: Into<Expr>,
    {
        Expr::Binary(Box::new(self), op, Box::new(rhs.into()))
    }

    /// Express a logical `AND` operation.
    pub fn and<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::And, right)
    }

    /// Express a logical `OR` operation.
    pub fn or<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Or, right)
    }

    /// Express an equal (`=`) expression.
    pub fn eq<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::Equal, right)
    }

    /// Express a not equal (`<>`) expression.
    pub fn ne<R>(self, right: R) -> Expr
    where
        R: Into<Expr>,
    {
        self.binary(BinaryOp::NotEqual, right)
    }

    /// Express a `IN` expression.
    pub fn is_in<V, I>(self, v: I) -> Expr
    where
        V: Into<Expr>,
        I: IntoIterator<Item = V>,
    {
        self.binary(
            BinaryOp::In,
            Expr::Tuple(v.into_iter().map(|v| v.into()).collect()),
        )
    }

    /// Express a `NOT IN` expression.
    pub fn is_not_in<V, I>(self, v: I) -> Expr
    where
        V: Into<Expr>,
        I: IntoIterator<Item = V>,
    {
        self.binary(
            BinaryOp::NotIn,
            Expr::Tuple(v.into_iter().map(|v| v.into()).collect()),
        )
    }

    /// Express a `IN` subquery expression.
    pub fn in_subquery(self, query: Select) -> Expr {
        self.binary(BinaryOp::In, Expr::SubQuery(None, Box::new(query)))
    }

    /// Apply any unary operator to the expression.
    pub fn unary(self, op: UnaryOp) -> Expr {
        Expr::Unary(op, Box::new(self))
    }

    /// Negates an expression with `NOT`.
    #[expect(clippy::should_implement_trait)]
    pub fn not(self) -> Expr {
        self.unary(UnaryOp::Not)
    }
}

/// SubQuery operators
#[derive(Debug, Copy, Clone, PartialEq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum SubQueryOp {
    Exists,
    Any,
    Some,
    All,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum UnaryOp {
    Not,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[expect(missing_docs)]
pub enum BinaryOp {
    And,
    Or,
    Equal,
    NotEqual,
    Between,
    NotBetween,
    Like,
    NotLike,
    Is,
    IsNot,
    In,
    NotIn,
    LShift,
    RShift,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl Expr {
    pub(crate) fn from_conditions(conditions: Vec<Expr>) -> Option<Expr> {
        conditions
            .into_iter()
            .reduce(|lhs, rhs| lhs.binary(BinaryOp::And, rhs))
    }
}

impl<T> From<T> for Expr
where
    T: Into<Value>,
{
    fn from(v: T) -> Self {
        Expr::Value(v.into())
    }
}

pub(crate) fn write_expr<W: SqlWriter>(w: &mut W, expr: &Expr) {
    match expr {
        Expr::Column(col) => write_column_ref(w, col),
        Expr::Asterisk => w.push_char('*'),
        Expr::Keyword(Keyword::Null) => w.push_str("NULL"),
        Expr::Tuple(exprs) => write_tuple(w, exprs),
        Expr::Value(value) => write_value(w, value.clone()),
        Expr::Unary(unary, expr) => write_unary_expr(w, unary, expr),
        Expr::Binary(lhs, op, rhs) => match (op, &**rhs) {
            (BinaryOp::In, Expr::Tuple(t)) if t.is_empty() => {
                // 1 = 2 is always false <=> IN () is always false
                write_binary_expr(w, &Expr::value(1), &BinaryOp::Equal, &Expr::value(2))
            }
            (BinaryOp::NotIn, Expr::Tuple(t)) if t.is_empty() => {
                // 1 = 1 is always true <=> NOT IN () is always true
                write_binary_expr(w, &Expr::value(1), &BinaryOp::Equal, &Expr::value(1))
            }
            _ => write_binary_expr(w, lhs, op, rhs),
        },
        Expr::FunctionCall(call) => write_function_call(w, call),
        Expr::SubQuery(op, query) => {
            if let Some(op) = op {
                w.push_str(match op {
                    SubQueryOp::Exists => "EXISTS",
                    SubQueryOp::Any => "ANY",
                    SubQueryOp::Some => "SOME",
                    SubQueryOp::All => "ALL",
                });
            }
            w.push_char('(');
            write_select(w, query);
            w.push_char(')');
        }
        Expr::Custom(expr) => w.push_str(expr),
    }
}

fn write_unary_expr<W: SqlWriter>(w: &mut W, op: &UnaryOp, expr: &Expr) {
    write_unary_op(w, op);
    w.push_char(' ');

    let mut paren = true;
    paren &= !well_known_no_parentheses(expr);
    paren &= !well_known_high_precedence(expr, &Operator::Unary(*op));
    if paren {
        w.push_char('(');
    }
    write_expr(w, expr);
    if paren {
        w.push_char(')');
    }
}

fn write_unary_op<W: SqlWriter>(w: &mut W, op: &UnaryOp) {
    w.push_str(match op {
        UnaryOp::Not => "NOT",
    })
}

fn write_binary_expr<W: SqlWriter>(w: &mut W, lhs: &Expr, op: &BinaryOp, rhs: &Expr) {
    let binop = Operator::Binary(*op);

    let mut left_paren = true;
    left_paren &= !well_known_no_parentheses(lhs);
    left_paren &= !well_known_high_precedence(lhs, &binop);
    // left associativity allow us to drop the left parentheses
    if left_paren
        && let Expr::Binary(_, inner_op, _) = lhs
        && inner_op == op
        && well_known_left_associative(op)
    {
        left_paren = false;
    }
    if left_paren {
        w.push_char('(');
    }
    write_expr(w, lhs);
    if left_paren {
        w.push_char(')');
    }

    w.push_char(' ');
    write_binary_op(w, op);
    w.push_char(' ');

    let mut right_paren = true;
    right_paren &= !well_known_no_parentheses(rhs);
    right_paren &= !well_known_high_precedence(rhs, &binop);
    // workaround represent trinary op between as nested binary ops
    if right_paren
        && binop.is_between()
        && let Expr::Binary(_, BinaryOp::And, _) = rhs
    {
        right_paren = false;
    }
    if right_paren {
        w.push_char('(');
    }
    write_expr(w, rhs);
    if right_paren {
        w.push_char(')');
    }
}

fn write_binary_op<W: SqlWriter>(w: &mut W, op: &BinaryOp) {
    w.push_str(match op {
        BinaryOp::And => "AND",
        BinaryOp::Or => "OR",
        BinaryOp::Like => "LIKE",
        BinaryOp::NotLike => "NOT LIKE",
        BinaryOp::Is => "IS",
        BinaryOp::IsNot => "IS NOT",
        BinaryOp::In => "IN",
        BinaryOp::NotIn => "NOT IN",
        BinaryOp::Between => "BETWEEN",
        BinaryOp::NotBetween => "NOT BETWEEN",
        BinaryOp::Equal => "=",
        BinaryOp::NotEqual => "<>",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::LShift => "<<",
        BinaryOp::RShift => ">>",
    })
}

fn write_tuple<W: SqlWriter>(w: &mut W, exprs: &[Expr]) {
    w.push_char('(');
    for (i, expr) in exprs.iter().enumerate() {
        if i != 0 {
            w.push_str(", ");
        }
        write_expr(w, expr);
    }
    w.push_char(')');
}

fn write_column_ref<W: SqlWriter>(w: &mut W, col: &ColumnRef) {
    match col {
        ColumnRef::Column(ColumnName(table_name, column)) => {
            if let Some(table_name) = table_name {
                write_table_name(w, table_name);
                w.push_char('.');
            }
            write_iden(w, column);
        }
        ColumnRef::Asterisk(table_name) => {
            if let Some(table_name) = table_name {
                write_table_name(w, table_name);
                w.push_char('.');
            }
            w.push_char('*');
        }
    }
}

fn well_known_no_parentheses(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Column(_)
            | Expr::Tuple(_)
            | Expr::Value(_)
            | Expr::Asterisk
            | Expr::Keyword(_)
            | Expr::FunctionCall(_)
            | Expr::SubQuery(_, _)
    )
}

fn well_known_left_associative(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::And
            | BinaryOp::Or
            | BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
    )
}

fn well_known_high_precedence(expr: &Expr, outer_op: &Operator) -> bool {
    let inner_op = if let Expr::Binary(_, op, _) = expr {
        Operator::Binary(*op)
    } else {
        return false;
    };

    if inner_op.is_arithmetic() || inner_op.is_shift() {
        return outer_op.is_comparison()
            || outer_op.is_between()
            || outer_op.is_in()
            || outer_op.is_like()
            || outer_op.is_logical();
    }

    if inner_op.is_comparison() || inner_op.is_in() || inner_op.is_like() || inner_op.is_is() {
        return outer_op.is_logical();
    }

    false
}

enum Operator {
    Unary(UnaryOp),
    Binary(BinaryOp),
}

impl Operator {
    fn is_logical(&self) -> bool {
        matches!(
            self,
            Operator::Unary(UnaryOp::Not)
                | Operator::Binary(BinaryOp::And)
                | Operator::Binary(BinaryOp::Or)
        )
    }

    fn is_between(&self) -> bool {
        matches!(
            self,
            Operator::Binary(BinaryOp::Between) | Operator::Binary(BinaryOp::NotBetween)
        )
    }

    fn is_like(&self) -> bool {
        matches!(
            self,
            Operator::Binary(BinaryOp::Like) | Operator::Binary(BinaryOp::NotLike)
        )
    }

    fn is_in(&self) -> bool {
        matches!(
            self,
            Operator::Binary(BinaryOp::In) | Operator::Binary(BinaryOp::NotIn)
        )
    }

    fn is_is(&self) -> bool {
        matches!(
            self,
            Operator::Binary(BinaryOp::Is) | Operator::Binary(BinaryOp::IsNot)
        )
    }

    fn is_shift(&self) -> bool {
        matches!(
            self,
            Operator::Binary(BinaryOp::LShift) | Operator::Binary(BinaryOp::RShift)
        )
    }

    fn is_arithmetic(&self) -> bool {
        match self {
            Operator::Binary(b) => {
                matches!(
                    b,
                    BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod | BinaryOp::Add | BinaryOp::Sub
                )
            }
            _ => false,
        }
    }

    fn is_comparison(&self) -> bool {
        match self {
            Operator::Binary(b) => {
                matches!(
                    b,
                    BinaryOp::LessThan
                        | BinaryOp::LessThanOrEqual
                        | BinaryOp::Equal
                        | BinaryOp::GreaterThanOrEqual
                        | BinaryOp::GreaterThan
                        | BinaryOp::NotEqual
                )
            }
            _ => false,
        }
    }
}
