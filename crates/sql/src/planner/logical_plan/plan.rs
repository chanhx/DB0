use {
    crate::{
        parser::ast::{Expr, TargetElem},
        planner::{PhysicalNode, Scan},
    },
    common::pub_fields_struct,
    def::JoinType,
};

#[derive(Debug)]
pub enum Node {
    Physical(PhysicalNode),

    Scan(Scan),
    Join {
        initial_node: Box<Node>,
        joined_nodes: Vec<JoinItem>,
    },
    Filter {
        input: Option<Box<Node>>,
        predict: Expr,
    },
    Projection {
        input: Option<Box<Node>>,
        distinct: bool,
        targets: Vec<TargetElem>,
    },
}

pub_fields_struct! {
    #[derive(Debug)]
    struct JoinItem {
        join_type: JoinType,
        node: Node,
        cond: Option<Expr>,
    }
}
