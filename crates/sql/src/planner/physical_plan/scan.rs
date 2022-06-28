use crate::{
    catalog::DatabaseCatalog,
    planner::{Node, Planner, Scan},
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn decide_scan_plan(&self, scan: Scan) -> Node {
        Node::SeqScan(scan)
    }
}
