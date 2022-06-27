mod logical_plan;

use crate::{
    catalog::{DatabaseCatalog, TableId, TableSchema},
    common::{macros::pub_fields_struct, JoinType},
    parser::ast::{Expr, TargetElem},
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
    CreateDatabase {
        if_not_exists: bool,
        name: String,
    },
    CreateTable {
        if_not_exists: bool,
        schema: TableSchema,
    },

    Scan(Scan),
    Filter(Filter),
    Projection(Projection),

    LogicalJoin {
        initial_node: Box<Node>,
        joined_nodes: Vec<JoinItem>,
    },

    Insert(Insert),
}

pub_fields_struct! {
    #[derive(Debug)]
    struct Scan {
        table_id: TableId,
        // TODO: use ColumnId
        projection: Option<Vec<String>>,
    }

    #[derive(Debug)]
    struct Filter {
        input: Option<Box<Node>>,
        predict: Expr,
    }

    #[derive(Debug)]
    struct Projection {
        input: Option<Box<Node>>,
        distinct: bool,
        targets: Vec<TargetElem>,
    }

    #[derive(Debug)]
    struct JoinItem {
        join_type: JoinType,
        node: Node,
        cond: Option<Expr>,
    }

    #[derive(Debug)]
    struct Insert {
        table_id: TableId,
        // TODO: use ColumnId
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expr>>,
    }
}
