use {
    super::Plan,
    def::{ColumnId, TableId},
    std::fmt::Display,
};

pub struct SortPlanNode {
    pub order_bys: Vec<OrderKey>,
    pub child: Plan,
}

pub struct OrderKey {
    pub table_id: TableId,
    pub column_id: ColumnId,
    pub order: SortingOrder,
}

pub enum SortingOrder {
    Ascending,
    Descending,
}

impl Display for SortPlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sort")
    }
}
