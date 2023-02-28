mod stmt;

use {
    binder::Binder, bound_ast::Statement, def::DatabaseId, snafu::prelude::*,
    storage::buffer::BufferManager,
};

#[derive(Debug, Snafu)]
pub enum Error {
    CreateTable { source: stmt::create_table::Error },

    Insert { source: stmt::insert::Error },

    Unspported,
}

pub struct Executor<'a> {
    database: DatabaseId,
    binder: &'a Binder,
}

impl Executor<'_> {
    pub fn execute(&self, stmt: Statement, manager: &mut BufferManager) -> Result<usize, Error> {
        match stmt {
            Statement::CreateTable(stmt) => {
                self.create_table(stmt, manager).context(CreateTableSnafu)
            }
            Statement::Insert(stmt) => self.insert(stmt, manager).context(InsertSnafu),
        }
    }
}
