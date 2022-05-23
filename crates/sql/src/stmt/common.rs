use crate::common::{macros, Span};

#[derive(Debug)]
pub struct Identifier(pub String, pub Span);

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Debug, PartialEq)]
pub enum DataType {
    Boolean,

    // Numeric types
    Bigint,
    Decimal,
    Float,
    Integer,
    SmallInt,

    // String types
    Char(u32),
    Varchar(u32),
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
    Unique {
        name: Option<Identifier>,
        columns: Vec<Identifier>,
    },
    PrimaryKey(Vec<Identifier>),
}

#[derive(Debug, PartialEq)]
pub enum Expr {}

macros::pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Column {
        name: Identifier,
        data_type: DataType,
        constraints: Vec<ColumnConstraint>,
    }

    #[derive(Debug, PartialEq)]
    struct Index {
        name: Identifier,
        columns: Vec<String>,
    }
}
