use {
    core::mem::transmute,
    snafu::prelude::*,
    std::{backtrace::Backtrace, convert::TryInto, fmt::Display},
};

#[derive(Debug, Snafu)]
pub enum Error {
    Encoding {
        ty: u8,
        len: i32,
        backtrace: Backtrace,
    },

    InvalidType {
        ty: u16,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! define_types {
    ($($var:ident$(($ty:ty))?,)*) => {
        #[derive(Debug, PartialEq, Clone, Hash)]
        pub enum DataType {
            $($var$(($ty))?,)*
        }

        #[derive(Debug, PartialEq, Clone, Hash)]
        #[repr(u16)]
        pub enum SqlType {
            $($var = ${index()} + 1,)*
        }

        impl TryInto<SqlType> for u16 {
            type Error = Error;

            fn try_into(self) -> Result<SqlType> {
                if self > 0 && self <= ${count(var)} {
                    Ok(unsafe { transmute::<u16, SqlType>(self) })
                } else {
                    Err(Error::InvalidType { ty: self })
                }
            }
        }
    };
}

define_types! {
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
    pub const fn value_repr(&self) -> (SqlType, u16) {
        match self {
            Self::Boolean => (SqlType::Boolean, 1),

            Self::TinyInt => (SqlType::TinyInt, 1),
            Self::SmallInt => (SqlType::SmallInt, 2),
            Self::Int => (SqlType::Int, 4),
            Self::BigInt => (SqlType::BigInt, 8),

            Self::TinyUint => (SqlType::TinyUint, 1),
            Self::SmallUint => (SqlType::SmallUint, 2),
            Self::Uint => (SqlType::Uint, 4),
            Self::BigUint => (SqlType::BigUint, 8),

            Self::Float => (SqlType::Float, 4),
            Self::Double => (SqlType::Double, 8),

            Self::Char(len) => (SqlType::Char, *len),
            Self::Varchar(len) => (SqlType::Varchar, *len),
        }
    }
}

impl SqlType {
    pub const fn is_variable_length(&self) -> bool {
        matches!(self, Self::Varchar)
    }
}

impl Display for SqlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Boolean => "boolean",
                Self::TinyInt => "tiny integer",
                Self::SmallInt => "small integer",
                Self::Int => "integer",
                Self::BigInt => "big integer",

                Self::TinyUint => "tiny unsigned integer",
                Self::SmallUint => "small unsigned integer",
                Self::Uint => "unsigned integer",
                Self::BigUint => "big integer",

                Self::Float => "float",
                Self::Double => "double",

                Self::Char => "char",
                Self::Varchar => "varying",
            }
        )
    }
}
