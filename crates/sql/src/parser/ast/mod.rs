mod common;
mod dml;
mod expr;

pub(crate) use self::{common::*, dml::*, expr::*};

use crate::common::Spanned;

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
        constraints: Vec<Spanned<TableConstraint>>,
        from_query: Option<Box<Query>>,
    },
    DropDatabase {
        name: Identifier,
    },
    DropTable {
        name: Identifier,
    },
    Insert {
        table: Identifier,
        columns: Option<Vec<Identifier>>,
        source: InsertSource,
    },
    Select(Query),
}
