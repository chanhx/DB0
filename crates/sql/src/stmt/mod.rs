mod common;
mod dml;

pub(crate) use {
    common::{Column, ColumnConstraint, DataType, Identifier, TableConstraint},
    dml::Select,
};

#[derive(Debug, PartialEq)]
pub enum Stmt {
    CreateDatabase {
        if_not_exists: bool,
        name: Identifier,
    },
    CreateIndex {
        is_unique: bool,
        name: Identifier,
        table: Identifier,
        columns: Vec<Identifier>,
    },
    CreateTable {
        if_not_exists: bool,
        name: Identifier,
        columns: Vec<Column>,
        constraints: Vec<TableConstraint>,
        from_query: Option<Box<Select>>,
    },
    DropDatabase {
        name: Identifier,
    },
    DropTable {
        name: Identifier,
    },
}
