//! Container for all SQL value types.

/// SQL value variants.
pub enum Value {
    /// A nullable boolean value.
    Bool(Option<bool>),
}
