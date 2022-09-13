use {
    super::{
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    crate::{
        ast::{dml::InsertSource, Statement},
        lexer::{Keyword, Token},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_insert(&mut self) -> Result<Statement> {
        self.must_match(Token::Keyword(Keyword::INTO))?;

        let table = self.parse_identifier()?;
        let columns = match self.tokens.peek() {
            Some(Ok((Token::LeftParen, _))) => {
                let (cols, _) =
                    self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;
                Some(cols)
            }
            _ => None,
        };

        let source = match_token!(self.tokens.next(), {
            (Token::Keyword(Keyword::VALUES), _) => {
                let values = self.parse_comma_separated(|parser|
                    parser.parse_comma_separated_within_parentheses(Self::parse_expr, false)
                )?
                .into_iter()
                .map(|(v, _)| v)
                .collect();

                InsertSource::Values(values)
            },
            (Token::Keyword(Keyword::SELECT), _) => {
                let select = self.parse_select()?;
                InsertSource::FromQuery(Box::new(select))
            },
        });

        Ok(Statement::Insert {
            table,
            columns,
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::ast::{dml::*, expr::*, identifier_from_str, ColumnRef},
    };

    #[test]
    fn it_works() {
        let input = "
            INSERT INTO abc(a, b, c) VALUES (1, 3.14, true);
            INSERT INTO def SELECT a, b FROM abc;
        ";
        let expected_output = vec![
            Ok(Statement::Insert {
                table: identifier_from_str("abc"),
                columns: Some(vec![
                    identifier_from_str("a"),
                    identifier_from_str("b"),
                    identifier_from_str("c"),
                ]),
                source: InsertSource::Values(vec![vec![
                    Expression::Literal(Literal::Integer(1)),
                    Expression::Literal(Literal::Float(3.14)),
                    Expression::Literal(Literal::Boolean(true)),
                ]]),
            }),
            Ok(Statement::Insert {
                table: identifier_from_str("def"),
                columns: None,
                source: InsertSource::FromQuery(Box::new(Query {
                    distinct: false,
                    targets: vec![
                        TargetElem::Expr {
                            expr: Expression::Column(ColumnRef {
                                column: identifier_from_str("a"),
                                table: None,
                            }),
                            alias: None,
                        },
                        TargetElem::Expr {
                            expr: Expression::Column(ColumnRef {
                                column: identifier_from_str("b"),
                                table: None,
                            }),
                            alias: None,
                        },
                    ],
                    from: Some(SelectFrom {
                        item: FromItem::Table {
                            name: identifier_from_str("abc"),
                            alias: None,
                        },
                        joins: vec![],
                    }),
                    cond: None,
                })),
            }),
        ];

        let output = Parser::parse(input);

        assert_eq!(output, expected_output);
    }
}
