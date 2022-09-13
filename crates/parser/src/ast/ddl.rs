use {
    super::{expr::Expression, Identifier, Query},
    crate::Spanned,
    common::pub_fields_struct,
    def::DataType,
};

#[derive(Debug, PartialEq)]
pub enum TableConstraint {
    Unique(Vec<Identifier>),
    PrimaryKey(Vec<Identifier>),
}

#[derive(Debug, PartialEq)]
pub enum ColumnConstraint {
    NotNull,
    PrimaryKey,
    Unique,
    Default(Expression),
}

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Column {
        name: Identifier,
        data_type: DataType,
        constraints: Vec<Spanned<ColumnConstraint>>,
    }

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
