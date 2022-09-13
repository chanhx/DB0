use {
    super::{expr::Expression, Identifier},
    common::pub_fields_struct,
    def::JoinType,
};

#[derive(Debug, PartialEq)]
pub enum FromItem {
    Table {
        name: Identifier,
        alias: Option<Identifier>,
    },
    SubQuery {
        query: Box<Query>,
        alias: Option<Identifier>,
    },
}

#[derive(Debug, PartialEq)]
pub enum TargetElem {
    Expr {
        expr: Expression,
        alias: Option<String>,
    },
    Wildcard {
        table: Option<String>,
    },
}

#[derive(Debug, PartialEq)]
pub enum InsertSource {
    Values(Vec<Vec<Expression>>),
    FromQuery(Box<Query>),
}

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Query {
        distinct: bool,
        targets: Vec<TargetElem>,
        from: Option<SelectFrom>,
        cond: Option<Expression>,
    }

    #[derive(Debug, PartialEq)]
    struct JoinItem {
        join_type: JoinType,
        item: FromItem,
        cond: Option<Expression>,
    }

    #[derive(Debug, PartialEq)]
    struct SelectFrom {
        item: FromItem,
        joins: Vec<JoinItem>,
    }
}
