mod stmt;

use {binder::Binder, bound_ast::Statement, snafu::prelude::*};

#[derive(Debug, Snafu)]
pub enum Error {
    CreateTable { source: stmt::create_table::Error },

    Insert { source: stmt::insert::Error },

    Unspported,
}

pub struct Analyzer<'a> {
    binder: &'a Binder,
}

impl Analyzer<'_> {
    pub fn analyze(&self, stmt: ast::Statement) -> Result<Statement, Error> {
        Ok(match stmt {
            ast::Statement::CreateTable(stmt) => {
                self.analyze_create_table(stmt).context(CreateTableSnafu)?
            }
            ast::Statement::Insert(stmt) => self.analyze_insert(stmt).context(InsertSnafu)?,
            _ => return Err(UnspportedSnafu.build()),
        })
    }
}
