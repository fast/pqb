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

use crate::expr::Expr;
use crate::expr::write_expr;
use crate::types::IntoColumnRef;
use crate::writer::SqlWriter;

/// Order expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    expr: Expr,
    direction: SortDirection,
    nulls: Option<NullOrdering>,
}

#[derive(Debug, Clone, PartialEq)]
enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
enum NullOrdering {
    First,
    Last,
}

impl Order {
    /// Order by a column in ascending order.
    pub fn column<T>(col: T) -> Self
    where
        T: IntoColumnRef,
    {
        Self {
            expr: Expr::column(col),
            direction: SortDirection::Asc,
            nulls: None,
        }
    }

    /// Order by an expression in ascending order.
    pub fn expr<E>(expr: E) -> Self
    where
        E: Into<Expr>,
    {
        let expr = expr.into();
        assert!(
            !matches!(
                expr,
                Expr::Value(_)
                    | Expr::Keyword(
                        crate::expr::Keyword::Null
                            | crate::expr::Keyword::True
                            | crate::expr::Keyword::False
                            | crate::expr::Keyword::Default
                    )
            ),
            "ORDER BY requires an expression or column; use Order::position for an output position"
        );
        Self {
            expr,
            direction: SortDirection::Asc,
            nulls: None,
        }
    }

    /// Order by a one-based output column position, rendered inline even in bound queries.
    pub fn position(position: u32) -> Self {
        assert!(
            position > 0 && position <= i32::MAX as u32,
            "ORDER BY position must be a positive int4"
        );
        Self::expr(Expr::custom(position.to_string()))
    }

    pub(super) fn valid_for_set(&self) -> bool {
        matches!(
            self.expr,
            Expr::Column(crate::types::ColumnRef::Column(crate::types::ColumnName(
                None,
                _
            ))) | Expr::Custom(_)
        )
    }

    /// Set the sort direction to ascending.
    pub fn asc(mut self) -> Self {
        self.direction = SortDirection::Asc;
        self
    }

    /// Set the sort direction to descending.
    pub fn desc(mut self) -> Self {
        self.direction = SortDirection::Desc;
        self
    }

    /// Set nulls to appear first.
    pub fn nulls_first(mut self) -> Self {
        self.nulls = Some(NullOrdering::First);
        self
    }

    /// Set nulls to appear last.
    pub fn nulls_last(mut self) -> Self {
        self.nulls = Some(NullOrdering::Last);
        self
    }
}

pub(crate) fn write_order<W: SqlWriter>(w: &mut W, order: &Order) {
    write_expr(w, &order.expr);
    match order.direction {
        SortDirection::Asc => w.push_str(" ASC"),
        SortDirection::Desc => w.push_str(" DESC"),
    }
    if let Some(nulls) = &order.nulls {
        match nulls {
            NullOrdering::First => w.push_str(" NULLS FIRST"),
            NullOrdering::Last => w.push_str(" NULLS LAST"),
        }
    }
}
