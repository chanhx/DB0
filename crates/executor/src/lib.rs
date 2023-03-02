mod stmt;

use {
    binder::Binder,
    bound_ast::Statement,
    def::{DatabaseId, Value},
    snafu::prelude::*,
    storage::buffer::BufferManager,
};

#[derive(Debug, Snafu)]
pub enum Error {
    CreateTable { source: stmt::CreateTableError },

    Insert { source: stmt::InsertError },

    Query { source: stmt::QueryError },

    Unspported,
}

pub struct Executor<'a> {
    database: DatabaseId,
    binder: &'a Binder,
}

impl Executor<'_> {
    pub fn execute(
        &self,
        stmt: Statement,
        manager: &mut BufferManager,
    ) -> Result<Vec<Vec<Value>>, Error> {
        match stmt {
            Statement::CreateTable(stmt) => {
                self.create_table(stmt, manager).context(CreateTableSnafu)
            }
            Statement::Insert(stmt) => self.insert(stmt, manager).context(InsertSnafu),
            Statement::Select(stmt) => self.select(stmt, manager).context(QuerySnafu),
        }
    }
}
