pub mod ddl;
pub mod dml;
pub mod expr;

use crate::common::*;

use self::{ddl::*, dml::*};

#[derive(Debug, PartialEq)]
pub enum Statement {
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
    CreateTable(CreateTableStmt),
    CreateTableAs(CreateTableAsStmt),
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
