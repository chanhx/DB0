mod join;
mod plan;
mod scan;

pub use plan::{Join, PhysicalNode};

use crate::{
    catalog::DatabaseCatalog,
    planner::{logical_plan::LogicalNode, Planner},
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub fn decide_physical_plan(&self, node: LogicalNode) -> PhysicalNode {
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
