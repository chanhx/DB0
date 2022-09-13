mod error;
// mod insert;
// mod query;
// mod table;

pub use error::{Error, Result};

use {super::LogicalNode, parser::ast::Statement};

struct Generator {}

impl Generator {
    fn new() -> Self {
        Self {}
    }

    fn generate_plan(&self, stmt: Statement) -> Result<LogicalNode> {
        Ok(match stmt {
            // Statement::Select(query) => self.build_query_plan(query)?,

            // Statement::Insert {
            //     table,
            //     columns,
            //     source,
            // } => self.build_insert(table.0, columns, source)?,
            _ => return Err(Error::Unimplemented),
        })
    }
}
