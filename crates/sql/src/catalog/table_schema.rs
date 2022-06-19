use crate::common::{macros::pub_fields_struct, DataType};

pub_fields_struct! {
    #[derive(Debug)]
    struct Column {
        name: String,
        data_type: DataType,
        is_nullable: bool,
        // default_value:
    }

    #[derive(Debug)]
    struct UniqueConstraint {
        name: String,
        columns: Vec<String>,
    }

    #[derive(Debug)]
    struct TableSchema {
        name: String,
        columns: Vec<Column>,
        primary_key_columns: Option<Vec<String>>,
        unique_contraints: Vec<UniqueConstraint>,
    }
}
