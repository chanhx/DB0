use {def::TableId, std::fmt::Display};

pub struct SeqScanPlanNode {
    pub table_id: TableId,
    pub table_name: String,
}

impl Display for SeqScanPlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Seq Scan on {}", self.table_name)
    }
}
