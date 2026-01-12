//!

use crate::value::Value;

///
pub trait SQLWriter {
    ///
    fn push_param(&mut self, value: Value);

    ///
    fn push_str(&mut self, value: &str);


    ///
    fn push_char(&mut self, value: char);
}

impl SQLWriter for String {
    fn push_param(&mut self, value: Value) {
        todo!()
    }

    fn push_str(&mut self, value: &str) {
        String::push_str(self, value)
    }

    fn push_char(&mut self, value: char) {
        String::push(self, value)
    }
}
