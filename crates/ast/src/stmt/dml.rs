use {
    crate::{common::Identifier, expr::Expression},
    common::pub_fields_struct,
    def::JoinType,
};

#[derive(Debug, PartialEq)]
pub enum TableFactor {
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
    struct InsertStmt{
        table: Identifier,
        targets: Option<Vec<Identifier>>,
        source: InsertSource,
    }

    #[derive(Debug, PartialEq)]
    struct Query {
        distinct: bool,
        targets: Vec<TargetElem>,
        from: Vec<TableReference>,
        cond: Option<Expression>,
    }

    #[derive(Debug, PartialEq)]
    struct JoinItem {
        join_type: JoinType,
        table_factor: TableFactor,
        cond: Option<Expression>,
    }

    #[derive(Debug, PartialEq)]
    struct TableReference {
        factor: TableFactor,
        joins: Vec<JoinItem>,
    }
}
