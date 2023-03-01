use {
    super::{
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    ast::{
        expr::*,
        token::{Keyword, Token},
        ColumnRef, Spanned,
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
            Ok(Spanned(t, _)) => T::from(t),
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
            Spanned(Token::Identifier, span) => {
                let id = self.identifier_from_span(span);

                match self.tokens.peek() {
                    Some(Ok(Spanned(Token::LeftParen, _))) => {
                        let Spanned(arguments, _) = self.parse_comma_separated_within_parentheses(Self::parse_expr, true)?;
                        Expression::FunctionCall { func: id, arguments }
                    },
                    Some(Ok(Spanned(Token::Period, _))) => {
                        self.tokens.next();
                        Expression::Column(ColumnRef {
                            name: self.parse_identifier()?,
                            table: Some(id),
                        })
                    },
                    _ => Expression::Column(ColumnRef {
                        name: id,
                        table: None,
                    })
                }
            },
            Spanned(Token::Number { is_float }, span) => {
                if is_float {
                    Literal::Float(self.number_from_span(span)?).into()
                } else {
                    Literal::Int(self.number_from_span(span)?).into()
                }
            },
            Spanned(Token::LeftParen, _) => {
                let expr = self.parse_expr()?;
                self.must_match(Token::RightParen)?;
                expr
            },
            Spanned(Token::String, span) => Literal::String(self.string_from_span(span)).into(),
            Spanned(Token::Keyword(Keyword::TRUE), _) => Literal::Boolean(true).into(),
            Spanned(Token::Keyword(Keyword::FALSE), _) => Literal::Boolean(false).into(),
            Spanned(Token::Keyword(Keyword::NULL), _) => Literal::Null.into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        ast::{expr::Operation, identifier_from_str},
    };

    #[test]
    fn it_works() {
        let input = "a + b.c + 1 >= +3.5";
        let expected_output = Expression::Operation(Operation::GreaterThanOrEqual(
            Box::new(Expression::Operation(Operation::Add(
                Box::new(Expression::Operation(Operation::Add(
                    Box::from(Expression::Column(ColumnRef {
                        name: identifier_from_str("a"),
                        table: None,
                    })),
                    Box::from(Expression::Column(ColumnRef {
                        name: identifier_from_str("c"),
                        table: Some(identifier_from_str("b")),
                    })),
                ))),
                Box::from(Expression::Literal(Literal::Int(1))),
            ))),
            Box::new(Expression::Operation(Operation::Positive(Box::new(
                Expression::Literal(Literal::Float(3.5)),
            )))),
        ));

        let output = Parser::new(input).parse_expr().unwrap();

        assert_eq!(output, expected_output);
    }
}
