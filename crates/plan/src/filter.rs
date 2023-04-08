use {super::Plan, bound_ast::Expression, std::fmt::Display};

pub struct FilterPlanNode {
    pub predicate: Expression,
    pub child: Plan,
}

impl Display for FilterPlanNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filter")
    }
}
