use {
    super::{
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    ast::{
        token::{Keyword, Token},
        InsertSource, InsertStmt, Spanned, Statement,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_insert(&mut self) -> Result<Statement> {
        self.must_match(Token::Keyword(Keyword::INTO))?;

        let table = self.parse_identifier()?;
        let targets = match self.tokens.peek() {
            Some(Ok(Spanned(Token::LeftParen, _))) => {
                let Spanned(cols, _) =
                    self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;
                Some(cols)
            }
            _ => None,
        };

        let source = match_token!(self.tokens.next(), {
            Spanned(Token::Keyword(Keyword::VALUES), _) => {
                let values = self.parse_comma_separated(|parser|
                    parser.parse_comma_separated_within_parentheses(Self::parse_expr, false)
                )?
                .into_iter()
                .map(|Spanned(v, _)| v)
                .collect();

                InsertSource::Values(values)
            },
            Spanned(Token::Keyword(Keyword::SELECT), _) => {
                let select = self.parse_select()?;
                InsertSource::FromQuery(Box::new(select))
            },
        });

        Ok(Statement::Insert(InsertStmt {
            table,
            targets,
            source,
        }))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        ast::{
            expr::*, identifier_from_str, ColumnRef, Query, TableFactor, TableReference, TargetElem,
        },
    };

    #[test]
    fn it_works() {
        let input = "
            INSERT INTO abc(a, b, c) VALUES (1, 3.14, true);
            INSERT INTO def SELECT a, b FROM abc;
        ";
        let expected_output = vec![
            Statement::Insert(InsertStmt {
                table: identifier_from_str("abc"),
                targets: Some(vec![
                    identifier_from_str("a"),
                    identifier_from_str("b"),
                    identifier_from_str("c"),
                ]),
                source: InsertSource::Values(vec![vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Float(3.14)),
                    Expression::Literal(Literal::Boolean(true)),
                ]]),
            }),
            Statement::Insert(InsertStmt {
                table: identifier_from_str("def"),
                targets: None,
                source: InsertSource::FromQuery(Box::new(Query {
                    distinct: false,
                    targets: vec![
                        TargetElem::Expr {
                            expr: Expression::Column(ColumnRef {
                                name: identifier_from_str("a"),
                                table: None,
                            }),
                            alias: None,
                        },
                        TargetElem::Expr {
                            expr: Expression::Column(ColumnRef {
                                name: identifier_from_str("b"),
                                table: None,
                            }),
                            alias: None,
                        },
                    ],
                    from: vec![TableReference {
                        factor: TableFactor::Table {
                            name: identifier_from_str("abc"),
                            alias: None,
                        },
                        joins: vec![],
                    }],
                    cond: None,
                })),
            }),
        ];

        let output = Parser::parse(input).unwrap();

        assert_eq!(output, expected_output);
    }
}
