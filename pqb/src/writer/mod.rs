use crate::value::Value;

mod query;
pub use self::query::write_select_statement;

mod value;
pub use self::value::write_value;

pub trait SqlWriter {
    fn push_param(&mut self, value: Value);

    fn push_str(&mut self, value: &str);

    fn push_char(&mut self, value: char);
}

impl SqlWriter for String {
    fn push_param(&mut self, value: Value) {
        write_value(self, &value);
    }

    fn push_str(&mut self, value: &str) {
        String::push_str(self, value)
    }

    fn push_char(&mut self, value: char) {
        String::push(self, value)
    }
}
