mod create_table;
mod insert;
mod plan;
mod query;

pub use plan::{JoinItem, Node};

use {
    super::error::{Error, Result},
    crate::{PhysicalNode, Planner},
    create_table::build_create_table_plan,
    def::catalog::DatabaseCatalog,
    parser::ast::Statement,
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn build_node(&self, stmt: Statement) -> Result<Node> {
        Ok(match stmt {
            Statement::Select(query) => self.build_query_plan(query)?,

            Statement::Insert {
                table,
                columns,
                source,
            } => self.build_insert(table.0, columns, source)?,

            _ => return Err(Error::Internal("unimplemented".to_string())),
        })
    }
}
