use crate::{
    common::{macros::pub_fields_struct, JoinType},
    parser::ast::{Expr, TargetElem},
    planner::Scan,
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
        predict: Expr,
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
        cond: Option<Expr>,
    }
}
