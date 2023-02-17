use {
    crate::{Insert, Scan},
    ast::{expr::Expression, TargetElem},
    common::pub_fields_struct,
    def::JoinType,
};

#[derive(Debug)]
pub enum PhysicalNode {
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
    struct Join {
        join_type: JoinType,
        left: Box<PhysicalNode>,
        right: Box<PhysicalNode>,
        cond: Option<Expression>,
    }
}
