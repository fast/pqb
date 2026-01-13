use crate::value::Value;
use crate::writer::SqlWriter;

pub fn write_value<W: SqlWriter>(w: &mut W, value: &Value) {
    match value {
        Value::Bool(None) => w.push_str("NULL"),
        Value::Bool(Some(b)) => w.push_str(if *b { "TRUE" } else { "FALSE" }),
    }
}
