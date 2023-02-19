mod stmt;

use {bound_ast::Statement, snafu::prelude::*};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("error in statement"))]
    Statement {
        source: Box<dyn snafu::Error>,
    },

    Unspported,
}

pub struct Analyzer {}

impl Analyzer {
    pub fn analyze(&self, stmt: ast::Statement) -> Result<Statement, Error> {
        Ok(match stmt {
            ast::Statement::CreateTable(stmt) => {
                self.analyze_create_table(stmt)
                    .map_err(|e| Error::Statement {
                        source: Box::new(e),
                    })?
            }
            _ => return Err(Error::Unspported),
        })
    }
}
