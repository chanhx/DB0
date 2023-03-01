use {
    crate::ColumnNum,
    common::pub_fields_struct,
    def::{TableId, Value},
};

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct InsertStmt {
        table: TableId,
        targets: Vec<ColumnNum>,
        // TODO: support expressions
        source: Vec<Vec<Value>>,
    }
}
