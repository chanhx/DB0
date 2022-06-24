use {
    super::{create_table::build_table_schema, Node},
    crate::{
        catalog::Catalog,
        error::{Error, Result},
        parser::ast::Stmt,
    },
};

pub struct Planner<'a, C: Catalog> {
    catalog: &'a mut C,
}

impl<'a, C: Catalog> Planner<'a, C> {
    pub fn new(catalog: &'a mut C) -> Self {
        Self { catalog }
    }

    pub fn build_node(&self, stmt: Stmt) -> Result<Node> {
        Ok(match stmt {
            Stmt::CreateDatabase {
                if_not_exists,
                name,
            } => Node::CreateDatabase {
                if_not_exists,
                name: name.0,
            },

            Stmt::CreateTable {
                if_not_exists,
                name,
                columns,
                constraints,
                from_query,
            } if from_query.is_none() => Node::CreateTable {
                if_not_exists,
                schema: build_table_schema(name.0, columns, constraints)?,
            },

            _ => return Err(Error::Internal("unimplemented".to_string())),
        })
    }
}
