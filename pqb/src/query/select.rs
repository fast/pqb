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
use crate::query::With;
use crate::query::order::Order;
use crate::query::order::write_order;
use crate::query::write_with;
use crate::types::Iden;
use crate::types::IntoColumnRef;
use crate::types::IntoIden;
use crate::types::IntoTableRef;
use crate::types::JoinType;
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_ref;
use crate::writer::SqlWriter;
use crate::writer::SqlWriterValues;

/// Select rows from an existing table.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Select {
    distinct: Option<Vec<Expr>>,
    set: Option<Box<SetOperation>>,
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
    table_sample: Option<TableSample>,
    with: Option<With>,
}

/// Join expression.
#[derive(Debug, Clone, PartialEq)]
pub struct JoinExpr {
    join_type: JoinType,
    table: TableRef,
    on: Option<JoinCondition>,
}

#[derive(Debug, Clone, PartialEq)]
enum JoinCondition {
    On(Expr),
    Using(Vec<Iden>),
}

#[derive(Debug, Clone, PartialEq)]
struct SetOperation {
    left: Select,
    op: SetOperator,
    right: Select,
}

/// Set operations combining two SELECT results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(missing_docs)]
pub enum SetOperator {
    Union,
    UnionAll,
    Intersect,
    IntersectAll,
    Except,
    ExceptAll,
}

impl Select {
    /// Create a new select statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Eliminate duplicate output rows.
    pub fn distinct(mut self) -> Self {
        self.distinct = Some(vec![]);
        self
    }

    /// Keep the first row of each group of expressions (DISTINCT ON).
    ///
    /// Start ORDER BY with these expressions to choose which row is kept.
    pub fn distinct_on<I, E>(mut self, expressions: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<Expr>,
    {
        self.distinct = Some(expressions.into_iter().map(Into::into).collect());
        self
    }

    /// Combine complete queries, preserving each operand's ordering and pagination.
    ///
    /// ORDER BY, LIMIT, OFFSET and WITH added afterwards apply to the combined result.
    /// To filter or project that result, wrap it with [`Self::from_subquery`].
    pub fn combine(self, op: SetOperator, right: Select) -> Self {
        Self {
            set: Some(Box::new(SetOperation {
                left: self,
                op,
                right,
            })),
            ..Self::new()
        }
    }

    /// Combine rows, eliminating duplicates.
    pub fn union(self, right: Select) -> Self {
        self.combine(SetOperator::Union, right)
    }

    /// Combine rows, retaining duplicates.
    pub fn union_all(self, right: Select) -> Self {
        self.combine(SetOperator::UnionAll, right)
    }

    /// Keep rows present in both queries, eliminating duplicates.
    pub fn intersect(self, right: Select) -> Self {
        self.combine(SetOperator::Intersect, right)
    }

    /// Remove rows present in the right query, eliminating duplicates.
    pub fn except(self, right: Select) -> Self {
        self.combine(SetOperator::Except, right)
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
            on: Some(JoinCondition::On(on.into())),
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
            on: Some(JoinCondition::On(on.into())),
        });
        self
    }

    /// Join a table or subquery with an ON condition.
    ///
    /// # Panics
    /// Panics for CROSS JOIN; use [`Self::cross_join`] instead.
    pub fn join<T, E>(mut self, join_type: JoinType, table: T, on: E) -> Self
    where
        T: IntoTableRef,
        E: Into<Expr>,
    {
        assert!(
            join_type != JoinType::CrossJoin,
            "use cross_join without an ON condition"
        );
        self.joins.push(JoinExpr {
            join_type,
            table: table.into(),
            on: Some(JoinCondition::On(on.into())),
        });
        self
    }

    /// Right outer join with an ON condition.
    pub fn right_join<T: IntoTableRef, E: Into<Expr>>(self, table: T, on: E) -> Self {
        self.join(JoinType::RightJoin, table, on)
    }

    /// Full outer join with an ON condition.
    pub fn full_outer_join<T: IntoTableRef, E: Into<Expr>>(self, table: T, on: E) -> Self {
        self.join(JoinType::FullOuterJoin, table, on)
    }

    /// Cartesian product with a table or subquery.
    pub fn cross_join<T: IntoTableRef>(mut self, table: T) -> Self {
        self.joins.push(JoinExpr {
            join_type: JoinType::CrossJoin,
            table: table.into(),
            on: None,
        });
        self
    }

    /// Join on equally named columns, emitting each join column once in SELECT *.
    ///
    /// # Panics
    /// Panics for an empty column list or a CROSS JOIN.
    pub fn join_using<T, I, C>(mut self, join_type: JoinType, table: T, columns: I) -> Self
    where
        T: IntoTableRef,
        I: IntoIterator<Item = C>,
        C: IntoIden,
    {
        let columns: Vec<_> = columns.into_iter().map(IntoIden::into_iden).collect();
        assert!(!columns.is_empty(), "USING requires at least one column");
        assert!(
            join_type != JoinType::CrossJoin,
            "CROSS JOIN cannot have USING"
        );
        self.joins.push(JoinExpr {
            join_type,
            table: table.into(),
            on: Some(JoinCondition::Using(columns)),
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

    /// Apply TABLESAMPLE clause.
    pub fn table_sample(mut self, table_sample: TableSample) -> Self {
        self.table_sample = Some(table_sample);
        self
    }

    /// WITH clause.
    pub fn with(mut self, with: With) -> Self {
        self.with = Some(with);
        self
    }
}

impl Select {
    pub(crate) fn columns_len(&self) -> Option<usize> {
        if let Some(set) = &self.set {
            return set.left.columns_len();
        }
        if self.selects.iter().any(|s| {
            matches!(
                s.expr,
                Expr::Asterisk
                    | Expr::Column(crate::types::ColumnRef::Asterisk(_))
                    | Expr::Custom(_)
                    | Expr::Fragment(_)
            )
        }) {
            return None;
        }
        Some(self.selects.len())
    }
}

/// Select expression used in select statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectExpr {
    expr: Expr,
    alias: Option<Iden>,
}

impl SelectExpr {
    /// Construct a projection expression without an alias.
    pub fn new(expr: impl Into<Expr>) -> Self {
        expr.into().into()
    }

    /// Set the output column name.
    pub fn alias(mut self, alias: impl IntoIden) -> Self {
        self.alias = Some(alias.into_iden());
        self
    }
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
            ty: RowLevelLockType::Update,
            tables: vec![],
            behavior: None,
        }
    }

    /// Create a new row-level lock for no key update.
    pub fn for_no_key_update() -> Self {
        Self {
            ty: RowLevelLockType::NoKeyUpdate,
            tables: vec![],
            behavior: None,
        }
    }

    /// Create a new row-level lock for share.
    pub fn for_share() -> Self {
        Self {
            ty: RowLevelLockType::Share,
            tables: vec![],
            behavior: None,
        }
    }

    /// Create a new row-level lock for key share.
    pub fn for_key_share() -> Self {
        Self {
            ty: RowLevelLockType::KeyShare,
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
    Update,
    NoKeyUpdate,
    Share,
    KeyShare,
}

/// Behavior of row-level locks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowLevelLockBehavior {
    Nowait,
    SkipLocked,
}

/// TABLESAMPLE clause.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableSample {
    method: SampleMethod,
    percentage: f64,
    repeatable: Option<f64>,
}

impl TableSample {
    /// Create a SYSTEM sampling clause.
    pub fn system() -> Self {
        Self {
            method: SampleMethod::SYSTEM,
            percentage: 100.0,
            repeatable: None,
        }
    }

    /// Create a BERNOULLI sampling clause.
    pub fn bernoulli() -> Self {
        Self {
            method: SampleMethod::BERNOULLI,
            percentage: 100.0,
            repeatable: None,
        }
    }

    /// Set the sampling percentage.
    ///
    /// The percentage should be a value between 0.0 and 100.0.
    pub fn percentage(mut self, percentage: f64) -> Self {
        self.percentage = percentage;
        self
    }

    /// Set the repeatable seed value.
    pub fn repeatable(mut self, repeatable: f64) -> Self {
        self.repeatable = Some(repeatable);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(clippy::upper_case_acronyms)]
enum SampleMethod {
    BERNOULLI,
    SYSTEM,
}

pub(crate) fn write_select<W: SqlWriter>(w: &mut W, select: &Select) {
    if let Some(with) = &select.with {
        write_with(w, with);
        w.push_char(' ');
    }

    if let Some(set) = &select.set {
        assert!(
            select.selects.is_empty()
                && select.from.is_empty()
                && select.joins.is_empty()
                && select.conditions.is_empty()
                && select.groups.is_empty()
                && select.having.is_empty()
                && select.distinct.is_none()
                && select.table_sample.is_none()
                && select.lock.is_none(),
            "wrap a set operation in from_subquery before projecting, filtering or locking it"
        );
        w.push_char('(');
        write_select(w, &set.left);
        w.push_str(") ");
        w.push_str(match set.op {
            SetOperator::Union => "UNION",
            SetOperator::UnionAll => "UNION ALL",
            SetOperator::Intersect => "INTERSECT",
            SetOperator::IntersectAll => "INTERSECT ALL",
            SetOperator::Except => "EXCEPT",
            SetOperator::ExceptAll => "EXCEPT ALL",
        });
        w.push_str(" (");
        write_select(w, &set.right);
        w.push_char(')');
        write_select_tail(w, select);
        return;
    }

    w.push_str("SELECT ");
    if let Some(distinct) = &select.distinct {
        w.push_str("DISTINCT ");
        if !distinct.is_empty() {
            w.push_str("ON ");
            crate::expr::write_tuple(w, distinct);
            w.push_char(' ');
        }
    }

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

    if let Some(table_sample) = &select.table_sample {
        match table_sample.method {
            SampleMethod::BERNOULLI => w.push_str(" TABLESAMPLE BERNOULLI"),
            SampleMethod::SYSTEM => w.push_str(" TABLESAMPLE SYSTEM"),
        }
        w.push_str(" (");
        w.push_fmt(format_args!("{}", table_sample.percentage));
        w.push_str(")");
        if let Some(repeatable) = table_sample.repeatable {
            w.push_str(" REPEATABLE (");
            w.push_fmt(format_args!("{repeatable}"));
            w.push_str(")");
        }
    }

    for join in &select.joins {
        match join.join_type {
            JoinType::LeftJoin => w.push_str(" LEFT JOIN "),
            JoinType::InnerJoin => w.push_str(" INNER JOIN "),
            JoinType::RightJoin => w.push_str(" RIGHT JOIN "),
            JoinType::FullOuterJoin => w.push_str(" FULL OUTER JOIN "),
            JoinType::CrossJoin => w.push_str(" CROSS JOIN "),
        }
        write_table_ref(w, &join.table);
        if let Some(on) = &join.on {
            match on {
                JoinCondition::On(expr) => {
                    w.push_str(" ON ");
                    write_expr(w, expr);
                }
                JoinCondition::Using(columns) => {
                    w.push_str(" USING (");
                    for (i, column) in columns.iter().enumerate() {
                        if i > 0 {
                            w.push_str(", ");
                        }
                        write_iden(w, column);
                    }
                    w.push_char(')');
                }
            }
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

    write_select_tail(w, select);
}

fn write_select_tail<W: SqlWriter>(w: &mut W, select: &Select) {
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
        w.push_fmt(format_args!("{limit}"));
    }

    if let Some(offset) = select.offset {
        w.push_str(" OFFSET ");
        w.push_fmt(format_args!("{offset}"));
    }

    if let Some(lock) = &select.lock {
        write_row_level_lock(w, lock);
    }
}

pub(crate) fn write_select_expr<W: SqlWriter>(w: &mut W, select_expr: &SelectExpr) {
    write_expr(w, &select_expr.expr);
    if let Some(alias) = &select_expr.alias {
        w.push_str(" AS ");
        write_iden(w, alias);
    }
}

fn write_row_level_lock<W: SqlWriter>(w: &mut W, lock: &RowLevelLock) {
    match lock.ty {
        RowLevelLockType::Update => w.push_str(" FOR UPDATE"),
        RowLevelLockType::NoKeyUpdate => w.push_str(" FOR NO KEY UPDATE"),
        RowLevelLockType::Share => w.push_str(" FOR SHARE"),
        RowLevelLockType::KeyShare => w.push_str(" FOR KEY SHARE"),
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
