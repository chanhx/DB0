use {
    super::{
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    ast::{
        expr::*,
        token::{Keyword, Token},
        FromItem, JoinItem, Query, SelectFrom, Spanned, TargetElem,
    },
    def::JoinType,
};

impl<'a> Parser<'a> {
    pub(super) fn parse_select(&mut self) -> Result<Query> {
        let distinct = self.try_match(Token::Keyword(Keyword::DISTINCT)).is_some();
        let targets = self.parse_select_targets()?;
        let from = self.parse_from_clause()?;
        let cond = self.parse_where_clause()?;

        Ok(Query {
            distinct,
            targets,
            from,
            cond,
        })
    }

    fn parse_select_targets(&mut self) -> Result<Vec<TargetElem>> {
        let targets = self.parse_comma_separated(Self::parse_select_target)?;
        Ok(targets)
    }

    fn parse_select_target(&mut self) -> Result<TargetElem> {
        Ok(match self.tokens.peek() {
            // TODO parse wildcard with table name
            Some(Ok(Spanned(Token::Asterisk, _))) => TargetElem::Wildcard { table: None },
            _ => {
                let expr = self.parse_expr()?;
                let alias = self.parse_alias()?.map(|id| id.0);
                TargetElem::Expr { expr, alias }
            }
        })
    }

    fn parse_from_clause(&mut self) -> Result<Option<SelectFrom>> {
        if self.try_match(Token::Keyword(Keyword::FROM)).is_none() {
            return Ok(None);
        };

        let from_item = self.parse_from_item()?;
        let mut joins = vec![];

        loop {
            match self.parse_join_item()? {
                Some(join) => joins.push(join),
                None => break,
            }
        }

        Ok(Some(SelectFrom {
            item: from_item,
            joins,
        }))
    }

    fn parse_from_item(&mut self) -> Result<FromItem> {
        Ok(match_token!(self.tokens.next(), {
            Spanned(Token::Identifier, span) => {
                let name = self.identifier_from_span(span);
                let alias = self.parse_alias()?;

                FromItem::Table { name, alias }
            },
            Spanned(Token::LeftParen, _) => {
                self.must_match(Token::Keyword(Keyword::SELECT))?;
                let subquery= self.parse_select()?;
                self.must_match(Token::RightParen)?;
                let alias = self.parse_alias()?;

                FromItem::SubQuery {
                    query: Box::new(subquery),
                    alias,
                }
            },
        }))
    }

    fn parse_join_item(&mut self) -> Result<Option<JoinItem>> {
        let join_type = match self.tokens.peek() {
            Some(Ok(Spanned(Token::Keyword(Keyword::JOIN), _))) => JoinType::Inner,
            Some(Ok(Spanned(Token::Keyword(Keyword::CROSS), _))) => {
                self.tokens.next();
                JoinType::Cross
            }
            Some(Ok(Spanned(Token::Keyword(Keyword::INNER), _))) => {
                self.tokens.next();
                JoinType::Inner
            }
            Some(Ok(Spanned(Token::Keyword(Keyword::LEFT), _))) => {
                self.tokens.next();
                JoinType::Left
            }
            Some(Ok(Spanned(Token::Keyword(Keyword::RIGHT), _))) => {
                self.tokens.next();
                JoinType::Right
            }
            _ => return Ok(None),
        };
        self.must_match(Token::Keyword(Keyword::JOIN))?;

        let item = self.parse_from_item()?;
        let cond = match self.try_match(Token::Keyword(Keyword::ON)) {
            Some(_) => Some(self.parse_expr()?),
            None => None,
        };

        Ok(Some(JoinItem {
            join_type,
            item,
            cond,
        }))
    }

    fn parse_where_clause(&mut self) -> Result<Option<Expression>> {
        Ok(match self.try_match(Token::Keyword(Keyword::WHERE)) {
            Some(_) => Some(self.parse_expr()?),
            None => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        ast::{identifier_from_str, ColumnRef, Statement},
    };

    #[test]
    fn it_works() {
        let input = "SELECT a, d.e AS de FROM abc INNER JOIN def as d ON abc.a = d.f WHERE c = 0;";
        let expected_output = vec![Statement::Select(Query {
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
                        column: identifier_from_str("e"),
                        table: Some(identifier_from_str("d")),
                    }),
                    alias: Some("de".into()),
                },
            ],
            from: Some(SelectFrom {
                item: FromItem::Table {
                    name: identifier_from_str("abc"),
                    alias: None,
                },
                joins: vec![JoinItem {
                    item: FromItem::Table {
                        name: identifier_from_str("def"),
                        alias: Some(identifier_from_str("d")),
                    },
                    join_type: JoinType::Inner,
                    cond: Some(Expression::Operation(Operation::Equal(
                        Box::new(Expression::Column(ColumnRef {
                            column: identifier_from_str("a"),
                            table: Some(identifier_from_str("abc")),
                        })),
                        Box::new(Expression::Column(ColumnRef {
                            column: identifier_from_str("f"),
                            table: Some(identifier_from_str("d")),
                        })),
                    ))),
                }],
            }),
            cond: Some(Expression::Operation(Operation::Equal(
                Box::new(Expression::Column(ColumnRef {
                    column: identifier_from_str("c"),
                    table: None,
                })),
                Box::new(Expression::Literal(Literal::Int(0))),
            ))),
        })];

        let output = Parser::parse(input).unwrap();

        assert_eq!(output, expected_output);
    }
}
