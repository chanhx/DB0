use {
    super::Expr,
    crate::common::{macros, DataType, Span, Spanned},
};

#[derive(Debug)]
pub struct Identifier(pub String, pub Span);

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
pub(crate) fn identifier_from_str(s: &str) -> Identifier {
    Identifier(s.to_string(), 0..=s.len() - 1)
}

#[derive(Debug, PartialEq)]
pub enum ColumnConstraint {
    NotNull,
    PrimaryKey,
    Unique,
    Default(Expr),
}

#[derive(Debug, PartialEq)]
pub enum TableConstraint {
    Unique(Vec<Identifier>),
    PrimaryKey(Vec<Identifier>),
}

macros::pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Column {
        name: Identifier,
        data_type: DataType,
        constraints: Vec<Spanned<ColumnConstraint>>,
    }

    #[derive(Debug, PartialEq)]
    struct ColumnRef {
        column: Identifier,
        table: Option<Identifier>,
    }

    #[derive(Debug, PartialEq)]
    struct Index {
        name: Identifier,
        columns: Vec<String>,
    }
}
