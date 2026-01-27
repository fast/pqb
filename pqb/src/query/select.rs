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
use crate::query::order::Order;
use crate::query::order::write_order;
use crate::types::Iden;
use crate::types::IntoColumnRef;
use crate::types::IntoIden;
use crate::types::IntoTableRef;
use crate::types::JoinType;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::value::Value;
use crate::value::write_value;
use crate::writer::SqlWriter;
use crate::writer::SqlWriterValues;

/// Select rows from an existing table.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Select {
    selects: Vec<SelectExpr>,
    from: Vec<TableRef>,
    joins: Vec<JoinExpr>,
    conditions: Vec<Expr>,
    groups: Vec<Expr>,
    having: Vec<Expr>,
    orders: Vec<Order>,
    limit: Option<u64>,
    offset: Option<u64>,
    lock: Option<RowLevelLock>,
}

/// Join expression.
#[derive(Debug, Clone, PartialEq)]
pub struct JoinExpr {
    join_type: JoinType,
    table: TableRef,
    on: Option<Expr>,
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

    /// Order by expressions.
    pub fn order_by<I>(mut self, orders: I) -> Self
    where
        I: IntoIterator<Item = Order>,
    {
        for order in orders {
            self.orders.push(order);
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

    /// Apply row-level lock.
    pub fn lock(mut self, lock: RowLevelLock) -> Self {
        self.lock = Some(lock);
        self
    }
}

impl Select {
    pub(crate) const fn columns_len(&self) -> usize {
        self.selects.len()
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

/// Row-level lock clause for select statements.
#[derive(Debug, Clone, PartialEq)]
pub struct RowLevelLock {
    ty: RowLevelLockType,
    tables: Vec<Iden>,
    behavior: Option<RowLevelLockBehavior>,
}

impl RowLevelLock {
    /// Create a new row-level lock for update.
    pub fn for_update() -> Self {
        Self {
            ty: RowLevelLockType::ForUpdate,
            tables: vec![],
            behavior: None,
        }
    }

    /// Create a new row-level lock for no key update.
    pub fn for_no_key_update() -> Self {
        Self {
            ty: RowLevelLockType::ForNoKeyUpdate,
            tables: vec![],
            behavior: None,
        }
    }

    /// Create a new row-level lock for share.
    pub fn for_share() -> Self {
        Self {
            ty: RowLevelLockType::ForShare,
            tables: vec![],
            behavior: None,
        }
    }

    /// Create a new row-level lock for key share.
    pub fn for_key_share() -> Self {
        Self {
            ty: RowLevelLockType::ForKeyShare,
            tables: vec![],
            behavior: None,
        }
    }

    /// Specify the lock behavior as NO WAIT.
    pub fn no_wait(mut self) -> Self {
        self.behavior = Some(RowLevelLockBehavior::Nowait);
        self
    }

    /// Specify the lock behavior as SKIP LOCKED.
    pub fn skip_locked(mut self) -> Self {
        self.behavior = Some(RowLevelLockBehavior::SkipLocked);
        self
    }

    /// Specify tables to apply the row-level lock.
    pub fn tables<T, I>(mut self, tables: I) -> Self
    where
        T: IntoIden,
        I: IntoIterator<Item = T>,
    {
        self.tables = tables.into_iter().map(|t| t.into_iden()).collect();
        self
    }
}

/// Types of row-level locks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowLevelLockType {
    ForUpdate,
    ForNoKeyUpdate,
    ForShare,
    ForKeyShare,
}

/// Behavior of row-level locks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowLevelLockBehavior {
    Nowait,
    SkipLocked,
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
            write_order(w, order);
        }
    }

    if let Some(limit) = select.limit {
        w.push_str(" LIMIT ");
        write_value(w, &Value::from(limit));
    }

    if let Some(offset) = select.offset {
        w.push_str(" OFFSET ");
        write_value(w, &Value::from(offset));
    }

    if let Some(lock) = &select.lock {
        write_row_level_lock(w, lock);
    }
}

fn write_select_expr<W: SqlWriter>(w: &mut W, select_expr: &SelectExpr) {
    write_expr(w, &select_expr.expr);
    if let Some(alias) = &select_expr.alias {
        w.push_str(" AS ");
        write_iden(w, alias);
    }
}

fn write_row_level_lock<W: SqlWriter>(w: &mut W, lock: &RowLevelLock) {
    match lock.ty {
        RowLevelLockType::ForUpdate => w.push_str(" FOR UPDATE"),
        RowLevelLockType::ForNoKeyUpdate => w.push_str(" FOR NO KEY UPDATE"),
        RowLevelLockType::ForShare => w.push_str(" FOR SHARE"),
        RowLevelLockType::ForKeyShare => w.push_str(" FOR KEY SHARE"),
    }

    if !lock.tables.is_empty() {
        w.push_str(" OF ");
        for (i, table) in lock.tables.iter().enumerate() {
            if i > 0 {
                w.push_str(", ");
            }
            write_iden(w, table);
        }
    }

    if let Some(behavior) = &lock.behavior {
        match behavior {
            RowLevelLockBehavior::Nowait => w.push_str(" NOWAIT"),
            RowLevelLockBehavior::SkipLocked => w.push_str(" SKIP LOCKED"),
        }
    }
}
