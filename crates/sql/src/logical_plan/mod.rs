mod create_table;
mod insert;
mod planner;
mod query;

pub use planner::Planner;

use crate::{
    catalog::{TableId, TableSchema},
    common::{macros::pub_fields_struct, JoinType},
    parser::ast::{Expr, TargetElem},
};

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
    Join(Join),

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
    struct Join {
        join_type: JoinType,
        left: Box<Node>,
        right: Box<Node>,
        // cond: Expr,
    }

    #[derive(Debug)]
    struct Insert {
        table_id: TableId,
        // TODO: use ColumnId
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expr>>,
    }
}
