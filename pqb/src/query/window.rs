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
use crate::query::Order;
use crate::query::write_order;
use crate::writer::SqlWriter;

/// An inline window specification for OVER (...).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Window {
    partitions: Vec<Expr>,
    orders: Vec<Order>,
    frame: Option<WindowFrame>,
}

impl Window {
    /// Create an empty window covering the whole result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append partition expressions.
    pub fn partition_by<I, E>(mut self, expressions: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<Expr>,
    {
        self.partitions
            .extend(expressions.into_iter().map(Into::into));
        self
    }

    /// Append ordering within each partition.
    pub fn order_by(mut self, orders: impl IntoIterator<Item = Order>) -> Self {
        self.orders.extend(orders);
        self
    }

    /// Set an explicit frame. Without this, PostgreSQL's default frame applies.
    pub fn frame(mut self, frame: WindowFrame) -> Self {
        self.frame = Some(frame);
        self
    }
}

/// A frame boundary. Offsets count rows, peer groups, or numeric range units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(missing_docs)]
pub enum FrameBound {
    UnboundedPreceding,
    Preceding(u64),
    CurrentRow,
    Following(u64),
    UnboundedFollowing,
}

/// Window frame units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameUnits {
    /// Count physical rows.
    Rows,
    /// Measure offsets using the ORDER BY value. Offsets require one numeric ordering expression.
    Range,
    /// Count peer groups. Requires ORDER BY.
    Groups,
}

/// An explicit BETWEEN frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowFrame {
    units: FrameUnits,
    start: FrameBound,
    end: FrameBound,
}

impl WindowFrame {
    /// Create a frame with the given units and bounds.
    ///
    /// # Panics
    /// Panics for bounds forbidden by PostgreSQL's frame grammar.
    pub fn new(units: FrameUnits, start: FrameBound, end: FrameBound) -> Self {
        fn position(bound: FrameBound) -> u8 {
            match bound {
                FrameBound::UnboundedPreceding => 0,
                FrameBound::Preceding(_) => 1,
                FrameBound::CurrentRow => 2,
                FrameBound::Following(_) => 3,
                FrameBound::UnboundedFollowing => 4,
            }
        }
        assert!(
            start != FrameBound::UnboundedFollowing
                && end != FrameBound::UnboundedPreceding
                && position(start) <= position(end),
            "invalid window frame boundaries"
        );
        Self { units, start, end }
    }

    /// Create a ROWS BETWEEN frame.
    pub fn rows(start: FrameBound, end: FrameBound) -> Self {
        Self::new(FrameUnits::Rows, start, end)
    }
}

pub(crate) fn write_window<W: SqlWriter>(w: &mut W, window: &Window) {
    w.push_str(" OVER (");
    if !window.partitions.is_empty() {
        w.push_str("PARTITION BY ");
        for (i, expr) in window.partitions.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            write_expr(w, expr);
        }
    }
    if !window.orders.is_empty() {
        if !window.partitions.is_empty() {
            w.push_char(' ');
        }
        w.push_str("ORDER BY ");
        for (i, order) in window.orders.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            write_order(w, order);
        }
    }
    if let Some(frame) = &window.frame {
        if !window.partitions.is_empty() || !window.orders.is_empty() {
            w.push_char(' ');
        }
        w.push_str(match frame.units {
            FrameUnits::Rows => "ROWS",
            FrameUnits::Range => "RANGE",
            FrameUnits::Groups => "GROUPS",
        });
        w.push_str(" BETWEEN ");
        write_bound(w, frame.start);
        w.push_str(" AND ");
        write_bound(w, frame.end);
    }
    w.push_char(')');
}

fn write_bound<W: SqlWriter>(w: &mut W, bound: FrameBound) {
    match bound {
        FrameBound::UnboundedPreceding => w.push_str("UNBOUNDED PRECEDING"),
        FrameBound::Preceding(n) => w.push_fmt(format_args!("{n} PRECEDING")),
        FrameBound::CurrentRow => w.push_str("CURRENT ROW"),
        FrameBound::Following(n) => w.push_fmt(format_args!("{n} FOLLOWING")),
        FrameBound::UnboundedFollowing => w.push_str("UNBOUNDED FOLLOWING"),
    }
}
