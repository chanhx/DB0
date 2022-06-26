use {
    super::{create_table::build_table_schema, Node},
    crate::{
        catalog::DatabaseCatalog,
        error::{Error, Result},
        parser::ast::Stmt,
    },
};

pub struct Planner<'a, D: DatabaseCatalog> {
    db_catalog: &'a mut D,
}

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub fn new(db_catalog: &'a mut D) -> Self {
        Self { db_catalog }
    }

    pub fn db_catalog(&self) -> &D {
        self.db_catalog
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
