use {bound_ast::Expression, def::TableId, std::fmt::Display};

pub struct IndexScanPlanNode {
    pub table_id: TableId,
    pub table_name: String,
    pub predicate: Expression,
}

impl Display for IndexScanPlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index Scan on {}", self.table_name)
    }
}
