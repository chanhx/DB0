mod stmt;

pub use stmt::*;

#[derive(Debug, PartialEq)]
pub enum Statement {
    CreateTable(CreateTableStmt),
}
