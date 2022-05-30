use {
    super::{Expr, Identifier},
    crate::common::macros,
};

#[derive(Debug, PartialEq)]
pub enum FromItem {
    Table(Identifier, Option<Identifier>),
    SubQuery(Box<Select>, Option<Identifier>),
}

#[derive(Debug, PartialEq)]
pub enum Join {
    Cross,
    Inner,
    Left,
    Right,
}

macros::pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Select {
        distinct: bool,
        targets: Vec<SelectItem>,
        from: Option<SelectFrom>,
        cond: Option<Expr>,
    }

    #[derive(Debug, PartialEq)]
    struct SelectItem {
        expr: Expr,
        alias: Option<Identifier>,
    }

    #[derive(Debug, PartialEq)]
    struct JoinItem {
        join: Join,
        item: FromItem,
        cond: Option<Expr>,
    }

    #[derive(Debug, PartialEq)]
    struct SelectFrom {
        item: FromItem,
        joins: Vec<JoinItem>,
    }
}
