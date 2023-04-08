mod arithmetic;
mod column;
mod comparison;
mod literal;
mod logic;

use {
    self::{arithmetic::ArithmeticExpression, comparison::ComparisonExpression},
    column::ColumnRef,
    def::{DataType, Value},
    literal::Literal,
    logic::LogicExpression,
    snafu::prelude::*,
};

#[derive(Debug)]
pub enum Expression {
    Column(ColumnRef),
    Literal(Literal),
    Logic(Box<LogicExpression>),
    Arithmetic(Box<ArithmeticExpression>),
    Comparison(Box<ComparisonExpression>),
    // IsNull,
    // Function,
}

#[derive(Debug, Snafu)]
pub enum Error {
    OperatorNotExists,
}

pub trait Evaluatate {
    fn return_type(&self) -> DataType;
    // TODO: it's unnecessary to provide all values in a tuple here
    fn evaluate(&self, values: &[Value]) -> Result<Value, Error>;
}

impl Evaluatate for Expression {
    fn return_type(&self) -> DataType {
        match self {
            Expression::Literal(expr) => expr.return_type(),
            Expression::Column(expr) => expr.return_type(),
            Expression::Logic(expr) => expr.return_type(),
            Expression::Arithmetic(expr) => expr.return_type(),
            Expression::Comparison(expr) => expr.return_type(),
        }
    }

    fn evaluate(&self, values: &[Value]) -> Result<Value, Error> {
        match self {
            Expression::Column(expr) => expr.evaluate(values),
            Expression::Literal(expr) => expr.evaluate(values),
            Expression::Logic(expr) => expr.evaluate(values),
            Expression::Arithmetic(expr) => expr.evaluate(values),
            Expression::Comparison(expr) => expr.evaluate(values),
        }
    }
}
