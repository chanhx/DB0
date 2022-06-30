use {
    super::PhysicalNode,
    crate::planner::{Planner, Scan},
    def::catalog::DatabaseCatalog,
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn decide_scan_plan(&self, scan: Scan) -> PhysicalNode {
        PhysicalNode::SeqScan(scan)
    }
}
