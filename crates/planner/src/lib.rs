mod error;
mod logical_plan;
mod physical_plan;

use {
    ast::expr::Expression,
    common::pub_fields_struct,
    def::catalog::{ColumnId, DatabaseCatalog, TableId},
    logical_plan::LogicalNode,
};
pub use {
    error::{Error, Result},
    physical_plan::PhysicalNode,
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

    // pub fn build_execution_plan(&self, stmt: Statement) -> Result<PhysicalNode> {
    //     let node = self.build_node(stmt)?;
    //     Ok(self.decide_physical_plan(node))
    // }
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
        columns: Option<Vec<ColumnId>>,
        values: Vec<Vec<Expression>>,
    }
}
