use crate::{
    catalog::TableSchema,
    common::{macros::pub_fields_struct, JoinType},
    parser::ast::{Expr, TargetElem},
    planner::{Insert, Scan},
};

#[derive(Debug)]
pub enum PhysicalNode {
    CreateDatabase {
        if_not_exists: bool,
        name: String,
    },
    CreateTable {
        if_not_exists: bool,
        schema: TableSchema,
    },

    SeqScan(Scan),
    IndexScan(Scan),
    IndexOnlyScan(Scan),

    Filter {
        input: Option<Box<PhysicalNode>>,
        predict: Expr,
    },
    Projection {
        input: Option<Box<PhysicalNode>>,
        distinct: bool,
        targets: Vec<TargetElem>,
    },

    HashJoin(Join),
    MergeJoin(Join),
    NestedLoopJoin(Join),

    Insert(Insert),
}

pub_fields_struct! {
    #[derive(Debug)]
    struct Join {
        join_type: JoinType,
        left: Box<PhysicalNode>,
        right: Box<PhysicalNode>,
        cond: Option<Expr>,
    }
}
