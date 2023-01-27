use super::Result;

#[derive(Debug, PartialEq)]
pub enum DataType {
    Boolean,

    // Numeric types
    TinyInt,
    SmallInt,
    Int,
    BigInt,

    TinyUint,
    SmallUint,
    Uint,
    BigUint,

    Float,
    Double,

    // String types
    Char(u16),
    Varchar(u16),
}

impl DataType {
    pub(super) fn new(ty: u8, len: i32) -> Result<Self> {
        Ok(match (ty, len) {
            (1, -1) => Self::Boolean,

            (2, -1) => Self::TinyInt,
            (3, -1) => Self::SmallInt,
            (4, -1) => Self::Int,
            (5, -1) => Self::BigInt,

            (6, -1) => Self::TinyUint,
            (7, -1) => Self::SmallUint,
            (8, -1) => Self::Uint,
            (9, -1) => Self::BigUint,

            (10, -1) => Self::Float,
            (11, -1) => Self::Double,

            (12, len) if len > 0 => Self::Char(len as u16),
            (13, len) if len > 0 => Self::Varchar(len as u16),

            _ => return Err(super::TypeEncodingSnafu { ty, len }.build()),
        })
    }

    pub fn is_variable_length(&self) -> bool {
        match self {
            Self::Varchar(_) => true,
            _ => false,
        }
    }

    pub(super) fn value_repr(&self) -> (u8, i32) {
        match self {
            Self::Boolean => (1, -1),

            Self::TinyInt => (2, -1),
            Self::SmallInt => (3, -1),
            Self::Int => (4, -1),
            Self::BigInt => (5, -1),

            Self::TinyUint => (6, -1),
            Self::SmallUint => (7, -1),
            Self::Uint => (8, -1),
            Self::BigUint => (9, -1),

            Self::Float => (10, -1),
            Self::Double => (11, -1),

            Self::Char(len) => (12, *len as i32),
            Self::Varchar(len) => (13, *len as i32),
        }
    }
}
