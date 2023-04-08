mod delete;
mod filter;
mod index_scan;
mod insert;
mod limit;
mod seq_scan;
mod sort;

pub use {
    delete::DeletePlanNode, filter::FilterPlanNode, index_scan::IndexScanPlanNode,
    insert::InsertPlanNode, limit::LimitPlanNode, seq_scan::SeqScanPlanNode, sort::SortPlanNode,
};

pub enum Plan {
    Delete(Box<DeletePlanNode>),
    Filter(Box<FilterPlanNode>),
    IndexScan(Box<IndexScanPlanNode>),
    Insert(Box<InsertPlanNode>),
    Limit(Box<LimitPlanNode>),
    SeqScan(Box<SeqScanPlanNode>),
    Sort(Box<SortPlanNode>),
}
