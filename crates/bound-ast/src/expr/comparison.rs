use {
    super::{Error, Evaluatate, Expression},
    def::{DataType, Value},
    std::cmp::Ordering,
};

#[derive(Debug)]
pub enum ComparisonExpression {
    Equal { lhs: Expression, rhs: Expression },
    NotEqual { lhs: Expression, rhs: Expression },
    LessThan { lhs: Expression, rhs: Expression },
    LessThanOrEqual { lhs: Expression, rhs: Expression },
    GreaterThan { lhs: Expression, rhs: Expression },
    GreaterThanOrEqual { lhs: Expression, rhs: Expression },
}

impl Evaluatate for ComparisonExpression {
    fn return_type(&self) -> DataType {
        DataType::Boolean
    }

    fn evaluate(&self, values: &[Value]) -> Result<Value, Error> {
        let (lhs, rhs) = match self {
            Self::Equal { lhs, rhs }
            | Self::NotEqual { lhs, rhs }
            | Self::LessThan { lhs, rhs }
            | Self::LessThanOrEqual { lhs, rhs }
            | Self::GreaterThan { lhs, rhs }
            | Self::GreaterThanOrEqual { lhs, rhs } => {
                (lhs.evaluate(values)?, rhs.evaluate(values)?)
            }
        };

        let ordering = match (lhs, rhs) {
            (Value::BigInt(lhs), Value::BigInt(rhs)) => lhs.cmp(&rhs),
            (Value::BigUint(lhs), Value::BigUint(rhs)) => lhs.cmp(&rhs),
            (Value::BigInt(lhs), Value::BigUint(rhs)) => lhs.cmp(&(rhs as i64)),
            (Value::BigUint(lhs), Value::BigInt(rhs)) => (lhs as i64).cmp(&rhs),
            _ => unreachable!(),
        };

        Ok(Value::Boolean(match self {
            Self::Equal { .. } => ordering == Ordering::Equal,
            Self::NotEqual { .. } => ordering != Ordering::Equal,
            Self::LessThan { .. } => ordering == Ordering::Less,
            Self::LessThanOrEqual { .. } => {
                ordering == Ordering::Less || ordering == Ordering::Equal
            }
            Self::GreaterThan { .. } => ordering == Ordering::Greater,
            Self::GreaterThanOrEqual { .. } => {
                ordering == Ordering::Greater || ordering == Ordering::Equal
            }
        }))
    }
}
