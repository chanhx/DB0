mod stmt;

use {
    binder::Binder,
    bound_ast::Statement,
    def::{DatabaseId, Value},
    snafu::prelude::*,
    std::sync::{Arc, RwLock},
    storage::buffer::BufferManager,
};

#[derive(Debug, Snafu)]
pub enum Error {
    CreateTable { source: stmt::CreateTableError },

    Insert { source: stmt::InsertError },

    Query { source: stmt::QueryError },

    Unspported,
}

pub struct Executor {
    database: DatabaseId,
    binder: Arc<RwLock<Binder>>,
}

impl Executor {
    pub fn new(database: DatabaseId, binder: Arc<RwLock<Binder>>) -> Self {
        Self { database, binder }
    }

    pub fn execute(
        &self,
        stmt: Statement,
        manager: &BufferManager,
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
