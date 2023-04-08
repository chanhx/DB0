use {super::Plan, def::TableId, std::fmt::Display};

pub struct InsertPlanNode {
    pub table_id: TableId,
    pub child: Plan,
}

impl Display for InsertPlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Insert")
    }
}
