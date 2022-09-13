use {
    super::{Column, Identifier, Query, TableConstraint},
    crate::Spanned,
    common::pub_fields_struct,
};

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct TableSchema {
        columns: Vec<Column>,
        constraints: Vec<Spanned<TableConstraint>>,
    }

    #[derive(Debug, PartialEq)]
    struct CreateTableStmt {
        if_not_exists: bool,
        name: Identifier,
        table_schema: TableSchema,
    }

    #[derive(Debug, PartialEq)]
    struct CreateTableAsStmt {
        if_not_exists: bool,
        name: Identifier,
        columns: Option<Vec<Identifier>>,
        constraints: Vec<Spanned<TableConstraint>>,
        query: Query,
    }
}
