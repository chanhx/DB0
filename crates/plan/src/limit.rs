use {super::Plan, std::fmt::Display};

pub struct LimitPlanNode {
    pub limit: usize,
    pub child: Plan,
}

impl Display for LimitPlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Limit (rows={})", self.limit)
    }
}
