use {super::Plan, def::TableId, std::fmt::Display};

pub struct DeletePlanNode {
    pub table_id: TableId,
    pub child: Plan,
}

impl Display for DeletePlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Delete {{ table_id={} }}", self.table_id)
    }
}
