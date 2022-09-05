use {
    super::{
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    crate::{
        ast::{ColumnRef, Expression, InfixOperator, Literal, Operator, PrefixOperator},
        lexer::{Keyword, Token},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self) -> Result<Expression> {
        self.parse_expr_recursive(0)
    }

    fn parse_expr_recursive(&mut self, min_prec: u8) -> Result<Expression> {
        let mut expr = match self.try_match_operator::<PrefixOperator>(min_prec) {
            Some(op) => op.build_expr(self.parse_expr_recursive(op.prec())?),
            None => self.parse_expr_atom()?,
        };

        while let Some(op) = self.try_match_operator::<InfixOperator>(min_prec) {
            expr = op.build_expr(expr, self.parse_expr_recursive(op.assoc() + op.prec())?);
        }

        Ok(expr)
    }

    fn try_match_operator<T: Operator>(&mut self, min_prec: u8) -> Option<T> {
        let op = match self.tokens.peek()? {
            Ok((t, _)) => T::from(t),
            _ => None,
        }
        .filter(|op| op.prec() >= min_prec);

        if op.is_some() {
            self.tokens.next();
        }

        op
    }

    fn parse_expr_atom(&mut self) -> Result<Expression> {
        Ok(match_token!(self.tokens.next(), {
            (Token::Identifier, span) => {
                let id = self.identifier_from_span(span);

                match self.tokens.peek() {
                    Some(Ok((Token::LeftParen, _))) => {
                        let (arguments, _) = self.parse_comma_separated_within_parentheses(Self::parse_expr, true)?;
                        Expression::FunctionCall { func: id, arguments }
                    },
                    Some(Ok((Token::Period, _))) => {
                        self.tokens.next();
                        let column = self.parse_identifier()?;
                        Expression::Column(ColumnRef {
                            column,
                            table: Some(id),
                        })
                    },
                    _ => Expression::Column(ColumnRef {
                        column: id,
                        table: None,
                    })
                }
            },
            (Token::Number { is_float }, span) => {
                if is_float {
                    Literal::Float(self.number_from_span(span)?).into()
                } else {
                    Literal::Integer(self.number_from_span(span)?).into()
                }
            },
            (Token::LeftParen, _) => {
                let expr = self.parse_expr()?;
                self.must_match(Token::RightParen)?;
                expr
            },
            (Token::String, span) => Literal::String(self.string_from_span(span)).into(),
            (Token::Keyword(Keyword::TRUE), _) => Literal::Boolean(true).into(),
            (Token::Keyword(Keyword::FALSE), _) => Literal::Boolean(false).into(),
            (Token::Keyword(Keyword::NULL), _) => Literal::Null.into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::ast::{identifier_from_str, Operation},
    };

    #[test]
    fn it_works() {
        let input = "a + b.c + 1 >= +3.5";
        let expected_output = Expression::Operation(Operation::GreaterThanOrEqual(
            Box::new(Expression::Operation(Operation::Add(
                Box::new(Expression::Operation(Operation::Add(
                    Box::from(Expression::Column(ColumnRef {
                        column: identifier_from_str("a"),
                        table: None,
                    })),
                    Box::from(Expression::Column(ColumnRef {
                        column: identifier_from_str("c"),
                        table: Some(identifier_from_str("b")),
                    })),
                ))),
                Box::from(Expression::Literal(Literal::Integer(1))),
            ))),
            Box::new(Expression::Operation(Operation::Positive(Box::new(
                Expression::Literal(Literal::Float(3.5)),
            )))),
        ));

        let output = Parser::new(input).parse_expr().unwrap();

        assert_eq!(output, expected_output);
    }
}
