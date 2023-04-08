mod expr;
mod stmt;

pub use {expr::*, stmt::*};

#[derive(Debug, PartialEq)]
pub enum Statement {
    CreateTable(CreateTableStmt),
    Insert(InsertStmt),
    Select(Query),
}
