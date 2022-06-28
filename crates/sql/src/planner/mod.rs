mod logical_plan;
mod physical_plan;

use {
    crate::{
        catalog::{DatabaseCatalog, TableId},
        common::macros::pub_fields_struct,
        parser::ast::Expr,
    },
    logical_plan::LogicalNode,
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
}

#[derive(Debug)]
pub enum Node {
    Logical(LogicalNode),
    Physical(PhysicalNode),
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
