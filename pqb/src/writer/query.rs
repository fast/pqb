use crate::query::SelectStatement;
use crate::writer::SqlWriter;

pub fn write_select_statement<W: SqlWriter>(w: &mut W, statement: &SelectStatement) {
    w.push_str("SELECT ");
}
