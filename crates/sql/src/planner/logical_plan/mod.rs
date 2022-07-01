mod create_table;
mod insert;
mod plan;
mod query;

pub use plan::{JoinItem, Node};

use {
    super::error::{Error, Result},
    crate::{
        parser::ast::Stmt,
        planner::{PhysicalNode, Planner},
    },
    create_table::build_create_table_plan,
    def::catalog::DatabaseCatalog,
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn build_node(&self, stmt: Stmt) -> Result<Node> {
        Ok(match stmt {
            Stmt::CreateDatabase {
                if_not_exists,
                name,
            } => Node::Physical(PhysicalNode::CreateDatabase {
                if_not_exists,
                name: name.0,
            }),

            Stmt::CreateTable {
                if_not_exists,
                name,
                columns,
                constraints,
                from_query,
            } if from_query.is_none() => Node::Physical(PhysicalNode::CreateTable(
                build_create_table_plan(if_not_exists, name.0, columns, constraints)?,
            )),

            Stmt::Select(query) => self.build_query_plan(query)?,

            Stmt::Insert {
                table,
                columns,
                source,
            } => self.build_insert(table.0, columns, source)?,

            _ => return Err(Error::Internal("unimplemented".to_string())),
        })
    }
}
