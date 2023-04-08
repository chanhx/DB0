use {
    super::{Error, Evaluatate, Expression},
    def::{DataType, Value},
};

#[derive(Debug)]
pub enum LogicExpression {
    And { lhs: Expression, rhs: Expression },
    Or { lhs: Expression, rhs: Expression },
    Not { child: Expression },
}

impl Evaluatate for LogicExpression {
    fn return_type(&self) -> DataType {
        DataType::Boolean
    }

    fn evaluate(&self, values: &[Value]) -> Result<Value, Error> {
        Ok(match self {
            Self::And { lhs, rhs } => match (lhs.evaluate(values)?, rhs.evaluate(values)?) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs & rhs),
                _ => unreachable!(),
            },
            Self::Or { lhs, rhs } => match (lhs.evaluate(values)?, rhs.evaluate(values)?) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs | rhs),
                _ => unreachable!(),
            },
            Self::Not { child } => match child.evaluate(values)? {
                Value::Boolean(val) => Value::Boolean(!val),
                _ => unreachable!(),
            },
        })
    }
}
