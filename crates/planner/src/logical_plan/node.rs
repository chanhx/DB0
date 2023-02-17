use {
    crate::Scan,
    ast::{expr::Expression, TargetElem},
    common::pub_fields_struct,
    def::JoinType,
};

#[derive(Debug)]
pub enum LogicalNode {
    Scan(Scan),
    Join {
        initial_node: Box<LogicalNode>,
        joined_nodes: Vec<JoinItem>,
    },
    Filter {
        input: Option<Box<LogicalNode>>,
        predict: Expression,
    },
    Projection {
        input: Option<Box<LogicalNode>>,
        distinct: bool,
        targets: Vec<TargetElem>,
    },
}

pub_fields_struct! {
    #[derive(Debug)]
    struct JoinItem {
        join_type: JoinType,
        node: LogicalNode,
        cond: Option<Expression>,
    }
}
