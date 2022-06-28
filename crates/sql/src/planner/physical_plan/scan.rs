use {
    super::PhysicalNode,
    crate::{
        catalog::DatabaseCatalog,
        planner::{Planner, Scan},
    },
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn decide_scan_plan(&self, scan: Scan) -> PhysicalNode {
        PhysicalNode::SeqScan(scan)
    }
}
