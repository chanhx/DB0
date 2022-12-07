use std::convert::TryFrom;

use snafu::{prelude::*, Backtrace};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("encoding error"))]
    Encoding {
        backtrace: Backtrace,
    },

    MismatchedType {
        backtrace: Backtrace,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

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
    Decimal,

    // String types
    Char(u16),
    Varchar(u16),
}

impl DataType {
    pub(super) fn from_value_repr(repr: (Value, Value)) -> Result<Self> {
        Ok(match repr {
            (Value::TinyUint(ty), Value::Int(len)) => match (ty, len) {
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
                (11, -1) => Self::Decimal,

                (12, len) if len > 0 => Self::Char(len as u16),
                (13, len) if len > 0 => Self::Varchar(len as u16),

                _ => return Err(EncodingSnafu.build()),
            },
            _ => return Err(EncodingSnafu.build()),
        })
    }

    pub(super) fn is_variable_length(&self) -> bool {
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
            Self::Decimal => (11, -1),

            Self::Char(len) => (12, *len as i32),
            Self::Varchar(len) => (13, *len as i32),
        }
    }
}

macro_rules! try_from_enum {
    ($name:ident, $var:ident, $t:ty) => {
        impl TryFrom<$name> for $t {
            type Error = Error;

            fn try_from(value: Value) -> Result<Self> {
                match value {
                    Value::$var(v) => Ok(v),
                    _ => Err(MismatchedTypeSnafu.build()),
                }
            }
        }
    };
    ($name:ident, $var:ident) => {};
}

macro_rules! auto_try_from {
    {
        $(#[$($attr:tt)*])*
        $vis:vis enum $name:ident {
            $($var:ident$(($ty:ty))?,)*
        }
    } => {
        $(#[$($attr)*])*
        $vis enum $name {
            $($var$(($ty))?,)*
        }

        $(try_from_enum!($name, $var$(, $ty)?);)*
    }
}

auto_try_from! {
    #[derive(Debug, PartialEq)]
    pub enum Value {
        Null,

        Boolean(bool),

        TinyInt(i8),
        SmallInt(i16),
        Int(i32),
        BigInt(i64),

        TinyUint(u8),
        SmallUint(u16),
        Uint(u32),
        BigUint(u64),

        Float(f32),
        Decimal(f64),

        String(String),
    }
}

impl Value {
    pub(super) fn byte_count(&self) -> usize {
        match self {
            Self::Null => 0,
            Self::Boolean(_) | Self::TinyInt(_) | Self::TinyUint(_) => 1,
            Self::SmallInt(_) | Self::SmallUint(_) => 2,
            Self::Int(_) | Self::Uint(_) | Self::Float(_) => 4,
            Self::BigInt(_) | Self::BigUint(_) | Self::Decimal(_) => 8,
            Self::String(s) => s.as_bytes().len(),
        }
    }
}

pub type Row = Vec<Value>;
