mod stmt;

use {
    binder::Binder,
    bound_ast::Statement,
    snafu::prelude::*,
    std::sync::{Arc, RwLock},
};

#[derive(Debug, Snafu)]
pub enum Error {
    CreateTable { source: stmt::CreateTableError },

    Insert { source: stmt::InsertError },

    Select { source: stmt::SelectError },

    Unspported,
}

pub struct Analyzer {
    binder: Arc<RwLock<Binder>>,
}

impl Analyzer {
    pub fn new(binder: Arc<RwLock<Binder>>) -> Self {
        Self { binder }
    }

    pub fn analyze(&self, stmt: ast::Statement) -> Result<Statement, Error> {
        Ok(match stmt {
            ast::Statement::CreateTable(stmt) => {
                self.analyze_create_table(stmt).context(CreateTableSnafu)?
            }
            ast::Statement::Insert(stmt) => self.analyze_insert(stmt).context(InsertSnafu)?,
            ast::Statement::Select(stmt) => self.analyze_select(stmt).context(SelectSnafu)?,
            _ => return Err(UnspportedSnafu.build()),
        })
    }
}
