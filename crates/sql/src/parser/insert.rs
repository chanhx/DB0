use {
    super::{
        ast::{InsertSource, Stmt},
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    crate::lexer::{Keyword, Token},
};

impl<'a> Parser<'a> {
    pub(super) fn parse_insert(&mut self) -> Result<Stmt> {
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

        Ok(Stmt::Insert {
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
        crate::parser::ast::{
            identifier_from_str, ColumnRef, Expr, FromItem, Literal, Query, SelectFrom, TargetElem,
        },
    };

    #[test]
    fn it_works() {
        let input = "
            INSERT INTO abc(a, b, c) VALUES (1, 3.14, true);
            INSERT INTO def SELECT a, b FROM abc;
        ";
        let expected_output = vec![
            Ok(Stmt::Insert {
                table: identifier_from_str("abc"),
                columns: Some(vec![
                    identifier_from_str("a"),
                    identifier_from_str("b"),
                    identifier_from_str("c"),
                ]),
                source: InsertSource::Values(vec![vec![
                    Expr::Literal(Literal::Integer(1)),
                    Expr::Literal(Literal::Float(3.14)),
                    Expr::Literal(Literal::Boolean(true)),
                ]]),
            }),
            Ok(Stmt::Insert {
                table: identifier_from_str("def"),
                columns: None,
                source: InsertSource::FromQuery(Box::new(Query {
                    distinct: false,
                    targets: vec![
                        TargetElem::Expr {
                            expr: Expr::Column(ColumnRef {
                                column: identifier_from_str("a"),
                                table: None,
                            }),
                            alias: None,
                        },
                        TargetElem::Expr {
                            expr: Expr::Column(ColumnRef {
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
