use crate::SqlWriterValues;
use crate::types::{Iden, IntoIden, TableRef, write_iden, write_table_ref};
use crate::writer::SqlWriter;

/// CREATE INDEX statement builder.
#[derive(Default, Debug, Clone)]
pub struct CreateIndex {
    table: Option<TableRef>,
    columns: Vec<Iden>,
}

impl CreateIndex {
    /// Create a new CREATE INDEX statement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the SQL string with placeholders and return collected values.
    pub fn to_values(&self) -> SqlWriterValues {
        let mut w = SqlWriterValues::new();
        write_create_index(&mut w, self);
        w
    }

    /// Convert the CREATE INDEX statement to a PostgreSQL query string.
    pub fn to_sql(&self) -> String {
        let mut sql = String::new();
        write_create_index(&mut sql, self);
        sql
    }

    /// Specify the table to create the index on.
    pub fn table<T>(mut self, table: T) -> Self
    where
        T: Into<TableRef>,
    {
        self.table = Some(table.into());
        self
    }

    /// Add a column to the index.
    pub fn column<T>(mut self, column: T) -> Self
    where
        T: IntoIden,
    {
        self.columns.push(column.into_iden());
        self
    }
}

fn write_create_index<W: SqlWriter>(w: &mut W, index: &CreateIndex) {
    w.push_str("CREATE INDEX ");
    w.push_str("ON ");
    if let Some(table) = &index.table {
        write_table_ref(w, table);
    }
    w.push_str(" (");
    for (i, col) in index.columns.iter().enumerate() {
        if i > 0 {
            w.push_str(", ");
        }
        write_iden(w, col);
    }
    w.push_str(")");
}
