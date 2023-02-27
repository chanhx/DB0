mod stmt;

use {bound_ast::Statement, def::DatabaseId, snafu::prelude::*, storage::buffer::BufferManager};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("error executing"))]
    Execution {
        source: Box<dyn snafu::Error>,
    },

    Unspported,
}

pub struct Executor {
    database: DatabaseId,
}

impl Executor {
    pub fn execute(&self, stmt: Statement, manager: &mut BufferManager) -> Result<usize, Error> {
        match stmt {
            Statement::CreateTable(stmt) => {
                self.create_table(stmt, manager)
                    .map_err(|e| Error::Execution {
                        source: Box::new(e),
                    })
            }
            Statement::Insert(_) => unimplemented!(),
        }
    }
}
