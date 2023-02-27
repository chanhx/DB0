//! The definition of `Expression` is directly inspired by `toydb`.
use crate::{
    common::{ColumnRef, Identifier},
    token::{Keyword, Token},
};

#[derive(Debug, PartialEq)]
pub enum Expression {
    Column(ColumnRef),
    Literal(Literal),
    FunctionCall {
        func: Identifier,
        arguments: Vec<Expression>,
    },
    Operation(Operation),
}

impl From<Literal> for Expression {
    fn from(literal: Literal) -> Self {
        Self::Literal(literal)
    }
}

impl From<Operation> for Expression {
    fn from(op: Operation) -> Self {
        Self::Operation(op)
    }
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Null,
    Boolean(bool),
    Int(i64),
    // TODO: update parsing logic
    Uint(u64),
    Float(f64),
    String(String),
}

pub trait Operator: Sized {
    /// Looks up the corresponding operator for a token, if one exists
    fn from(token: &Token) -> Option<Self>;

    /// Returns the operator's associativity
    fn assoc(&self) -> u8;

    /// Returns the operator's precedence
    fn prec(&self) -> u8;
}

macro_rules! op_variants {
    ($((unary, $($uop:ident)*))* $((binary, $($bop:ident)*))*) => {
        #[derive(Debug, PartialEq)]
        pub enum Operation {
            $($($uop(Box<Expression>),)*)*
            $($($bop(Box<Expression>, Box<Expression>),)*)*
        }
    }
}

macro_rules! build_expr {
    (unary { $($op:ident)* }) => {
        pub fn build_expr(&self, expr: Expression) -> Expression {
            let expr = Box::new(expr);

            match self {
                $( Self::$op => Operation::$op(expr), )*
            }
            .into()
        }
    };
    (binary { $($op:ident)* }) => {
        pub fn build_expr(&self, lhs: Expression, rhs: Expression) -> Expression {
            let (lhs, rhs) = (Box::new(lhs), Box::new(rhs));

            match self {
                $( Self::$op => Operation::$op(lhs, rhs), )*
            }
            .into()
        }
    };
}

macro_rules! operations {
    {
        $(
            $ty:ident $id:ident {
                $( ($op:ident, ($token:pat), $prec:literal, $assoc:literal), )*
            }
        )*
    }
    => {
        op_variants!($(($ty, $($op)*))*);

        $(
            #[derive(Debug, PartialEq)]
            pub enum $id {
                $( $op, )*
            }

            impl $id {
                build_expr!($ty {$($op)*});
            }

            impl Operator for $id {
                fn from(token: &Token) -> Option<Self> {
                    Some(match token {
                        $( $token => Self::$op, )*
                        _ => return None,
                    })
                }

                fn assoc(&self) -> u8 {
                    match self {
                        $( Self::$op => $assoc, )*
                    }
                }

                fn prec(&self) -> u8 {
                    match self {
                        $( Self::$op => $prec, )*
                    }
                }
            }
        )*
    }
}

operations!(
    unary PrefixOperator {
        (Positive, (Token::Plus), 9, 1),
        (Not, (Token::Keyword(Keyword::NOT)), 9, 1),
        (Negative, (Token::Minus), 9, 1),
    }

    binary InfixOperator {
        (And, (Token::Keyword(Keyword::AND)), 2, 1),
        (Or, (Token::Keyword(Keyword::OR)), 1, 1),

        (Is, (Token::Keyword(Keyword::IS)), 8, 1),
        (Equal, (Token::Equal), 3, 1),
        (NotEqual, (Token::NotEqual | Token::LessOrGreaterThan), 3, 1),
        (GreaterThan, (Token::GreaterThan), 4, 1),
        (GreaterThanOrEqual, (Token::GreaterThanOrEqual), 4, 1),
        (LessThan, (Token::LessThan), 4, 1),
        (LessThanOrEqual, (Token::LessThanOrEqual), 4, 1),

        (Add, (Token::Plus), 5, 1),
        (Subtract, (Token::Minus), 5, 1),
        (Multiply, (Token::Asterisk), 6, 1),
        (Divide, (Token::Slash), 6, 1),
        (Modulo, (Token::Percent), 6, 1),
        (Exponentiate, (Token::Caret), 7, 0),

        (Like, (Token::Keyword(Keyword::LIKE)), 3, 1),
    }
);
