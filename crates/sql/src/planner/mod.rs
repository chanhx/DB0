mod logical_plan;
mod physical_plan;

pub use physical_plan::PhysicalNode;

use {
    crate::{
        catalog::{DatabaseCatalog, TableId},
        common::macros::pub_fields_struct,
        error::Result,
        parser::ast::{Expr, Stmt},
    },
    logical_plan::Node,
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

    pub fn build_execution_plan(&self, stmt: Stmt) -> Result<PhysicalNode> {
        let node = self.build_node(stmt)?;
        Ok(self.decide_physical_plan(node))
    }
}

pub_fields_struct! {
    #[derive(Debug)]
    struct Scan {
        table_id: TableId,
        // TODO: use ColumnId
        projection: Option<Vec<String>>,
    }

    #[derive(Debug)]
    struct Insert {
        table_id: TableId,
        // TODO: use ColumnId
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expr>>,
    }
}
