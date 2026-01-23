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
use crate::types::TableRef;
use crate::types::write_iden;
use crate::types::write_table_name;
use crate::writer::SqlWriter;

/// Select rows from an existing table.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Select {
    selects: Vec<SelectExpr>,
    from: Vec<TableRef>,
}

impl Select {
    /// From table.
    pub fn from(mut self, table: impl Into<TableRef>) -> Self {
        self.from.push(table.into());
        self
    }

    /// Add an expression to the select expression list.
    pub fn expr<T>(&mut self, expr: T) -> &mut Self
    where
        T: Into<SelectExpr>,
    {
        self.selects.push(expr.into());
        self
    }

    /// Convert the select statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_select_statement(&mut sql, self);
        sql
    }
}

impl Select {
    /// Create a new select statement.
    pub fn new() -> Self {
        Self::default()
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

pub(crate) fn write_select_statement<W: SqlWriter>(w: &mut W, statement: &Select) {
    w.push_str("SELECT ");

    for (i, select_expr) in statement.selects.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_select_expr(w, select_expr);
    }

    for (i, table_ref) in statement.from.iter().enumerate() {
        if i == 0 {
            w.push_str(" FROM ");
        } else {
            w.push_str(", ");
        }
        match table_ref {
            TableRef::Table(table_name, alias) => {
                write_table_name(w, table_name);
                if let Some(alias) = alias {
                    w.push_str(" AS ");
                    write_iden(w, alias);
                }
            }
        }
    }
}

fn write_select_expr<W: SqlWriter>(w: &mut W, select_expr: &SelectExpr) {
    write_expr(w, &select_expr.expr);
    if let Some(alias) = &select_expr.alias {
        w.push_str(" AS ");
        write_iden(w, alias);
    }
}
