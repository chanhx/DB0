use {
    crate::ColumnNum,
    common::pub_fields_struct,
    def::{DataType, SchemaId},
};

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Column {
        name: String,
        data_type: DataType,
        is_nullable: bool,
        // Default(Expression),
    }

    #[derive(Debug, PartialEq)]
    struct CreateTableStmt {
        if_not_exists: bool,
        // database: DatabaseId,
        schema: SchemaId,
        name: String,
        columns: Vec<Column>,
        primary_key: Option<Vec<ColumnNum>>,
        unique_constraints: Vec<Vec<ColumnNum>>,
    }
}
