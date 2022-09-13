use {
    super::{Join, PhysicalNode},
    crate::{logical_plan::JoinItem, LogicalNode, Planner},
    def::catalog::DatabaseCatalog,
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn decide_join_plan(
        &self,
        initial_node: LogicalNode,
        joined_nodes: Vec<JoinItem>,
    ) -> PhysicalNode {
        let mut node = self.decide_physical_plan(initial_node);

        for join in joined_nodes {
            let joined_node = self.decide_physical_plan(join.node);

            node = PhysicalNode::NestedLoopJoin(Join {
                join_type: join.join_type,
                left: Box::new(node),
                right: Box::new(joined_node),
                cond: join.cond,
            })
        }

        node
    }
}
