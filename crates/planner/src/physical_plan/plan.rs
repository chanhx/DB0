use {
    crate::{Insert, Scan},
    common::pub_fields_struct,
    def::{
        catalog::{ColumnDef, UniqueConstraint},
        JoinType,
    },
    parser::ast::{Expression, TargetElem},
};

#[derive(Debug)]
pub enum PhysicalNode {
    CreateDatabase {
        if_not_exists: bool,
        name: String,
    },
    CreateTable(CreateTable),

    SeqScan(Scan),
    IndexScan(Scan),
    IndexOnlyScan(Scan),

    Filter {
        input: Option<Box<PhysicalNode>>,
        predict: Expression,
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
    struct CreateTable {
        if_not_exists: bool,
        name: String,
        columns: Vec<ColumnDef>,
        primary_key_columns: Option<Vec<String>>,
        unique_contraints: Vec<UniqueConstraint>,
    }

    #[derive(Debug)]
    struct Join {
        join_type: JoinType,
        left: Box<PhysicalNode>,
        right: Box<PhysicalNode>,
        cond: Option<Expression>,
    }
}
