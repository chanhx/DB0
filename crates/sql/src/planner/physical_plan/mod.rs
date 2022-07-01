mod join;
mod plan;
mod scan;

pub use self::plan::*;

use {
    crate::planner::{Node, Planner},
    def::catalog::DatabaseCatalog,
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn decide_physical_plan(&self, node: Node) -> PhysicalNode {
        match node {
            Node::Physical(node) => node,

            Node::Scan(scan) => self.decide_scan_plan(scan),

            Node::Join {
                initial_node,
                joined_nodes,
            } => self.decide_join_plan(*initial_node, joined_nodes),

            Node::Filter { input, predict } => {
                let input = input.map(|input| Box::new(self.decide_physical_plan(*input)));

                PhysicalNode::Filter { input, predict }
            }

            Node::Projection {
                input,
                distinct,
                targets,
            } => {
                let input = input.map(|input| Box::new(self.decide_physical_plan(*input)));

                PhysicalNode::Projection {
                    input,
                    distinct,
                    targets,
                }
            }
        }
    }
}
