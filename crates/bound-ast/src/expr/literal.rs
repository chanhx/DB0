use {
    super::{Error, Evaluatate},
    def::{DataType, Value},
};

#[derive(Debug, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Int(i64),
    Uint(u64),
    Float(f64),
    // String(String),
}

impl Evaluatate for Literal {
    fn return_type(&self) -> DataType {
        match self {
            Self::Boolean(_) => DataType::Boolean,
            Self::Int(_) => DataType::Int,
            Self::Uint(_) => DataType::Uint,
            Self::Float(_) => DataType::Float,
            // Self::String(_) => DataType::String,
        }
    }

    fn evaluate(&self, _: &[Value]) -> Result<Value, Error> {
        Ok(match self {
            Self::Boolean(v) => Value::Boolean(*v),
            Self::Int(v) => Value::BigInt(*v),
            Self::Uint(v) => Value::BigUint(*v),
            Self::Float(v) => Value::Double(*v),
            // Self::String(v) => Value::String(v.clone()),
        })
    }
}
