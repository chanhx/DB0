mod create_table;
mod insert;
mod plan;
mod query;

pub use plan::{JoinItem, LogicalNode};

use {
    crate::{
        catalog::DatabaseCatalog,
        error::{Error, Result},
        parser::ast::Stmt,
        planner::{Node, PhysicalNode, Planner},
    },
    create_table::build_table_schema,
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub fn build_node(&self, stmt: Stmt) -> Result<Node> {
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
            } if from_query.is_none() => Node::Physical(PhysicalNode::CreateTable {
                if_not_exists,
                schema: build_table_schema(name.0, columns, constraints)?,
            }),

            Stmt::Select(query) => Node::Logical(self.build_query_plan(query)?),

            Stmt::Insert {
                table,
                columns,
                source,
            } => self.build_insert(table.0, columns, source)?,

            _ => return Err(Error::Internal("unimplemented".to_string())),
        })
    }
}
