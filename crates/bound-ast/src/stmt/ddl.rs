use {
    common::pub_fields_struct,
    def::{DataType, SchemaId, TableId, Value},
};

pub type ColumnNum = i16;

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

    #[derive(Debug, PartialEq)]
    struct InsertStmt {
        table: TableId,
        targets: Vec<ColumnNum>,
        // TODO: support expressions
        source: Vec<Vec<Value>>,
    }
}
