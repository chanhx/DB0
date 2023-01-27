mod types;
mod value;

use {
    common::pub_fields_struct,
    snafu::{prelude::*, Backtrace},
    std::{convert::TryInto, io, string::FromUtf8Error},
};
pub use {
    types::DataType,
    value::{Row, Value},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(self)))]
pub enum Error {
    FromIo {
        source: io::Error,
    },

    Utf8Encoding {
        source: FromUtf8Error,
    },

    #[snafu(display("internal error"))]
    Internal {
        backtrace: Backtrace,
    },

    MismatchedType {
        backtrace: Backtrace,
    },

    TypeEncoding {
        ty: u8,
        len: i32,
        backtrace: Backtrace,
    },

    #[snafu(display("the count of values does not match the count of attributes"))]
    ValuesCount {
        backtrace: Backtrace,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Attribute {
        name: String,
        num: i16,
        data_type: DataType,
        is_nullable: bool,
    }
}

impl Attribute {
    pub fn new(name: String, num: i16, data_type: DataType, is_nullable: bool) -> Self {
        Self {
            name,
            num,
            data_type,
            is_nullable,
        }
    }

    pub fn to_values(&self) -> Vec<Value> {
        let (ty, len) = self.data_type.value_repr();

        vec![
            Value::String(self.name.clone()),
            Value::SmallInt(self.num),
            Value::TinyUint(ty),
            Value::Int(len),
            Value::Boolean(self.is_nullable),
        ]
    }
}

impl TryFrom<Vec<Value>> for Attribute {
    type Error = Error;

    fn try_from(values: Vec<Value>) -> Result<Self> {
        let values: [Value; 5] = values.try_into().map_err(|_| InternalSnafu.build())?;

        match values {
            [Value::String(name), Value::SmallInt(num), Value::TinyUint(ty), Value::Int(len), Value::Boolean(is_nullable)] => {
                Ok(Self {
                    name,
                    num,
                    data_type: DataType::new(ty, len)?,
                    is_nullable,
                })
            }
            _ => Err(InternalSnafu.build()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_attribute() {
        let meta_attributes = [
            Attribute::new("name".to_string(), 0, DataType::Varchar(20), false),
            Attribute::new("num".to_string(), 1, DataType::SmallInt, false),
            Attribute::new("type".to_string(), 2, DataType::TinyInt, false),
            Attribute::new("length".to_string(), 3, DataType::Int, false),
            Attribute::new("is_nullable".to_string(), 3, DataType::Boolean, false),
        ];

        meta_attributes.iter().for_each(|attr| {
            let values = attr.to_values();
            let attr_from_values: Attribute = values.try_into().unwrap();

            assert_eq!(*attr, attr_from_values);
        })
    }
}
