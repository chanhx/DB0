use {
    super::{Error, Evaluatate, Expression},
    def::{DataType, Value},
};

#[derive(Debug)]
pub enum ArithmeticExpression {
    Plus { lhs: Expression, rhs: Expression },
    Minus { lhs: Expression, rhs: Expression },
    Multiply { lhs: Expression, rhs: Expression },
    Divide { lhs: Expression, rhs: Expression },
    Positive { child: Expression },
    Negative { child: Expression },
}

impl ArithmeticExpression {
    fn calculate(
        &self,
        lhs: &Expression,
        rhs: &Expression,
        values: &[Value],
    ) -> Result<Value, Error> {
        let (lhs, rhs) = match (lhs.evaluate(values)?, rhs.evaluate(values)?) {
            values @ (Value::BigInt(_), Value::BigInt(_)) => values,
            (Value::BigUint(lhs), Value::BigUint(rhs)) => {
                (Value::BigInt(lhs as i64), Value::BigInt(rhs as i64))
            }
            (Value::BigInt(lhs), Value::BigUint(rhs)) => {
                (Value::BigInt(lhs), Value::BigInt(rhs as i64))
            }
            (Value::BigUint(lhs), Value::BigInt(rhs)) => {
                (Value::BigInt(lhs as i64), Value::BigInt(rhs))
            }
            _ => unreachable!(),
        };

        Ok(match (self, lhs, rhs) {
            (Self::Plus { .. }, Value::BigInt(lhs), Value::BigInt(rhs)) => Value::BigInt(lhs + rhs),
            (Self::Minus { .. }, Value::BigInt(lhs), Value::BigInt(rhs)) => {
                Value::BigInt(lhs - rhs)
            }
            (Self::Multiply { .. }, Value::BigInt(lhs), Value::BigInt(rhs)) => {
                Value::BigInt(lhs * rhs)
            }
            (Self::Divide { .. }, Value::BigInt(lhs), Value::BigInt(rhs)) => {
                Value::BigInt(lhs / rhs)
            }
            _ => unreachable!(),
        })
    }
}

impl Evaluatate for ArithmeticExpression {
    fn return_type(&self) -> DataType {
        DataType::BigInt
    }

    fn evaluate(&self, values: &[Value]) -> Result<Value, Error> {
        // TOOD: handle overflow problem
        Ok(match self {
            Self::Positive { child } => match child.evaluate(values)? {
                v @ (Value::BigInt(_) | Value::BigUint(_)) => v,
                _ => unreachable!(),
            },
            Self::Negative { child } => match child.evaluate(values)? {
                Value::BigInt(v) => Value::BigInt(-v),
                Value::BigUint(v) => Value::BigInt(-(v as i64)),
                _ => unreachable!(),
            },
            Self::Plus { lhs, rhs }
            | Self::Minus { lhs, rhs }
            | Self::Multiply { lhs, rhs }
            | Self::Divide { lhs, rhs } => self.calculate(lhs, rhs, values)?,
        })
    }
}
