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
use crate::types::Iden;
use crate::types::IntoColumnRef;
use crate::types::IntoIden;
use crate::types::IntoTableRef;
use crate::types::JoinType;
use crate::types::Order;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::value::write_value;
use crate::writer::{SqlWriter, SqlWriterValues};

/// Select rows from an existing table.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Select {
    selects: Vec<SelectExpr>,
    from: Vec<TableRef>,
    joins: Vec<JoinExpr>,
    conditions: Vec<Expr>,
    groups: Vec<Expr>,
    having: Vec<Expr>,
    orders: Vec<OrderExpr>,
    limit: Option<u64>,
    offset: Option<u64>,
}

/// Join expression.
#[derive(Debug, Clone, PartialEq)]
pub struct JoinExpr {
    join_type: JoinType,
    table: TableRef,
    on: Option<Expr>,
}

/// Order expression.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderExpr {
    expr: Expr,
    order: Order,
}

impl Select {
    /// Create a new select statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_select(&mut w, self);
        w
    }

    /// Convert the select statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_select(&mut sql, self);
        sql
    }

    /// From table.
    pub fn from<R>(mut self, table: R) -> Self
    where
        R: IntoTableRef,
    {
        self.from.push(table.into());
        self
    }

    /// From table with alias.
    pub fn from_as<R, A>(mut self, table: R, alias: A) -> Self
    where
        R: IntoTableRef,
        A: IntoIden,
    {
        self.from.push(table.into().alias(alias.into_iden()));
        self
    }

    /// From sub-query.
    pub fn from_subquery<T>(mut self, query: Select, alias: T) -> Self
    where
        T: IntoIden,
    {
        self.from
            .push(TableRef::SubQuery(query.into(), alias.into_iden()));
        self
    }

    /// Add an expression to the select expression list.
    pub fn expr<T>(mut self, expr: T) -> Self
    where
        T: Into<SelectExpr>,
    {
        self.selects.push(expr.into());
        self
    }

    /// Add a function to the select expression list.
    pub fn func<F>(mut self, func: F) -> Self
    where
        F: Into<Expr>,
    {
        self.selects.push(SelectExpr {
            expr: func.into(),
            alias: None,
        });
        self
    }

    /// Add an expression to the select expression list with its alias.
    pub fn expr_as<T, A>(mut self, expr: T, alias: A) -> Self
    where
        T: Into<Expr>,
        A: IntoIden,
    {
        self.selects.push(SelectExpr {
            expr: expr.into(),
            alias: Some(alias.into_iden()),
        });
        self
    }

    /// Add select expressions.
    pub fn exprs<T, I>(mut self, exprs: I) -> Self
    where
        T: Into<SelectExpr>,
        I: IntoIterator<Item = T>,
    {
        for expr in exprs {
            self.selects.push(expr.into());
        }
        self
    }

    /// Add a column to the select expression list.
    pub fn column<C>(self, col: C) -> Self
    where
        C: IntoColumnRef,
    {
        self.expr(Expr::Column(col.into_column_ref()))
    }

    /// Select columns.
    pub fn columns<T, I>(self, cols: I) -> Self
    where
        T: IntoColumnRef,
        I: IntoIterator<Item = T>,
    {
        self.exprs(cols.into_iter().map(|c| Expr::Column(c.into_column_ref())))
    }

    /// And where condition.
    pub fn and_where<T>(mut self, expr: T) -> Self
    where
        T: Into<Expr>,
    {
        self.conditions.push(expr.into());
        self
    }

    /// Left join with another table.
    pub fn left_join<T, E>(mut self, table: T, on: E) -> Self
    where
        T: IntoTableRef,
        E: Into<Expr>,
    {
        self.joins.push(JoinExpr {
            join_type: JoinType::LeftJoin,
            table: table.into(),
            on: Some(on.into()),
        });
        self
    }

    /// Inner join with another table.
    pub fn inner_join<T, E>(mut self, table: T, on: E) -> Self
    where
        T: IntoTableRef,
        E: Into<Expr>,
    {
        self.joins.push(JoinExpr {
            join_type: JoinType::InnerJoin,
            table: table.into(),
            on: Some(on.into()),
        });
        self
    }

    /// Order by column.
    pub fn order_by<C>(mut self, col: C, order: Order) -> Self
    where
        C: IntoColumnRef,
    {
        self.orders.push(OrderExpr {
            expr: Expr::Column(col.into_column_ref()),
            order,
        });
        self
    }

    /// Order by multiple columns.
    pub fn order_by_columns<T, I>(mut self, cols: I) -> Self
    where
        T: IntoColumnRef,
        I: IntoIterator<Item = (T, Order)>,
    {
        for (col, order) in cols {
            self.orders.push(OrderExpr {
                expr: Expr::Column(col.into_column_ref()),
                order,
            });
        }
        self
    }

    /// GROUP BY columns.
    pub fn group_by_columns<T, I>(mut self, cols: I) -> Self
    where
        T: IntoColumnRef,
        I: IntoIterator<Item = T>,
    {
        for col in cols {
            self.groups.push(Expr::Column(col.into_column_ref()));
        }
        self
    }

    /// GROUP BY expressions.
    pub fn group_by_exprs<T, I>(mut self, exprs: I) -> Self
    where
        T: Into<Expr>,
        I: IntoIterator<Item = T>,
    {
        for expr in exprs {
            self.groups.push(expr.into());
        }
        self
    }

    /// HAVING condition.
    pub fn and_having<T>(mut self, expr: T) -> Self
    where
        T: Into<Expr>,
    {
        self.having.push(expr.into());
        self
    }

    /// Offset number of returned rows.
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Limit the number of returned rows.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Select expression used in select statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectExpr {
    expr: Expr,
    alias: Option<Iden>,
}

impl<T> From<T> for SelectExpr
where
    T: Into<Expr>,
{
    fn from(expr: T) -> Self {
        SelectExpr {
            expr: expr.into(),
            alias: None,
        }
    }
}

pub(crate) fn write_select<W: SqlWriter>(w: &mut W, select: &Select) {
    w.push_str("SELECT ");

    for (i, select_expr) in select.selects.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_select_expr(w, select_expr);
    }

    for (i, table_ref) in select.from.iter().enumerate() {
        if i == 0 {
            w.push_str(" FROM ");
        } else {
            w.push_str(", ");
        }
        write_table_ref(w, table_ref);
    }

    for join in &select.joins {
        match join.join_type {
            JoinType::LeftJoin => w.push_str(" LEFT JOIN "),
            JoinType::InnerJoin => w.push_str(" INNER JOIN "),
        }
        write_table_ref(w, &join.table);
        if let Some(on) = &join.on {
            w.push_str(" ON ");
            write_expr(w, on);
        }
    }

    if let Some(condition) = Expr::from_conditions(select.conditions.clone()) {
        w.push_str(" WHERE ");
        write_expr(w, &condition);
    }

    if !select.groups.is_empty() {
        w.push_str(" GROUP BY ");
        for (i, group) in select.groups.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            write_expr(w, group);
        }
    }

    if let Some(having) = Expr::from_conditions(select.having.clone()) {
        w.push_str(" HAVING ");
        write_expr(w, &having);
    }

    if !select.orders.is_empty() {
        w.push_str(" ORDER BY ");
        for (i, order) in select.orders.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            write_expr(w, &order.expr);
            match order.order {
                Order::Asc => w.push_str(" ASC"),
                Order::Desc => w.push_str(" DESC"),
            }
        }
    }

    if let Some(limit) = select.limit {
        w.push_str(" LIMIT ");
        write_value(w, limit.into());
    }

    if let Some(offset) = select.offset {
        w.push_str(" OFFSET ");
        write_value(w, offset.into());
    }
}

fn write_select_expr<W: SqlWriter>(w: &mut W, select_expr: &SelectExpr) {
    write_expr(w, &select_expr.expr);
    if let Some(alias) = &select_expr.alias {
        w.push_str(" AS ");
        write_iden(w, alias);
    }
}
