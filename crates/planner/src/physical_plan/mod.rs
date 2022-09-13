mod join;
mod node;
mod scan;

pub use self::node::*;

use {
    crate::{LogicalNode, Planner},
    def::catalog::DatabaseCatalog,
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn decide_physical_plan(&self, node: LogicalNode) -> PhysicalNode {
        match node {
            LogicalNode::Scan(scan) => self.decide_scan_plan(scan),

            LogicalNode::Join {
                initial_node,
                joined_nodes,
            } => self.decide_join_plan(*initial_node, joined_nodes),

            LogicalNode::Filter { input, predict } => {
                let input = input.map(|input| Box::new(self.decide_physical_plan(*input)));

                PhysicalNode::Filter { input, predict }
            }

            LogicalNode::Projection {
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
