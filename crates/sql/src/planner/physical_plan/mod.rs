mod join;
mod scan;

use crate::{
    catalog::DatabaseCatalog,
    planner::{Node, Planner},
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub fn decide_physical_plan(&self, node: Node) -> Node {
        match node {
            Node::LogicalJoin {
                initial_node,
                joined_nodes,
            } => self.decide_join_plan(*initial_node, joined_nodes),

            Node::LogicalScan(scan) => self.decide_scan_plan(scan),

            Node::Projection(mut projection) => {
                projection.input = projection
                    .input
                    .map(|input| Box::new(self.decide_physical_plan(*input)));

                Node::Projection(projection)
            }
            Node::Filter(mut filter) => {
                filter.input = filter
                    .input
                    .map(|input| Box::new(self.decide_physical_plan(*input)));

                Node::Filter(filter)
            }

            other => other,
        }
    }
}
