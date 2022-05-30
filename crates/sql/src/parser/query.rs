use {
    super::{common::match_token, Parser},
    crate::{
        error::{Error, Result},
        lexer::{Keyword, Token},
        stmt::{Expr, FromItem, Join, JoinItem, Select, SelectFrom, SelectItem},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_select(&mut self) -> Result<Select> {
        let distinct = self.try_match(Token::Keyword(Keyword::DISTINCT)).is_some();
        let targets = self.parse_select_targets()?;
        let from = self.parse_from_clause()?;
        let cond = self.parse_where_clause()?;

        Ok(Select {
            distinct,
            targets,
            from,
            cond,
        })
    }

    fn parse_select_targets(&mut self) -> Result<Vec<SelectItem>> {
        let targets = self.parse_comma_separated(Self::parse_select_target)?;
        Ok(targets)
    }

    fn parse_select_target(&mut self) -> Result<SelectItem> {
        let expr = self.parse_expr()?;
        let alias = self.parse_alias()?;

        Ok(SelectItem { expr, alias })
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
            (Token::Identifier, span) => {
                let table = self.identifier_from_span(span);
                let alias = self.parse_alias()?;

                FromItem::Table(table, alias)
            },
            (Token::LeftParen, _) => {
                self.must_match(Token::Keyword(Keyword::SELECT))?;
                let subquery= self.parse_select()?;
                self.must_match(Token::RightParen)?;
                let alias = self.parse_alias()?;

                FromItem::SubQuery(
                    Box::new(subquery),
                    alias,
                )
            },
        }))
    }

    fn parse_join_item(&mut self) -> Result<Option<JoinItem>> {
        let join = match self.tokens.peek() {
            Some(Ok((Token::Keyword(Keyword::JOIN), _))) => Join::Inner,
            Some(Ok((Token::Keyword(Keyword::CROSS), _))) => {
                self.tokens.next();
                Join::Cross
            }
            Some(Ok((Token::Keyword(Keyword::INNER), _))) => {
                self.tokens.next();
                Join::Inner
            }
            Some(Ok((Token::Keyword(Keyword::LEFT), _))) => {
                self.tokens.next();
                Join::Left
            }
            Some(Ok((Token::Keyword(Keyword::RIGHT), _))) => {
                self.tokens.next();
                Join::Right
            }
            _ => return Ok(None),
        };
        self.must_match(Token::Keyword(Keyword::JOIN))?;

        let item = self.parse_from_item()?;
        let cond = match self.try_match(Token::Keyword(Keyword::ON)) {
            Some(_) => Some(self.parse_expr()?),
            None => None,
        };

        Ok(Some(JoinItem { join, item, cond }))
    }

    fn parse_where_clause(&mut self) -> Result<Option<Expr>> {
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
        crate::{
            common::test_utils::identifier_from_str,
            stmt::{ColumnIdentifier, Literal, Operation, Stmt},
        },
    };

    #[test]
    fn it_works() {
        let input = "SELECT a, d.e AS de FROM abc INNER JOIN def as d ON abc.a = d.f WHERE c = 0;";
        let expected_output = vec![Ok(Stmt::Select(Select {
            distinct: false,
            targets: vec![
                SelectItem {
                    expr: Expr::Column(ColumnIdentifier {
                        column: identifier_from_str("a"),
                        table: None,
                    }),
                    alias: None,
                },
                SelectItem {
                    expr: Expr::Column(ColumnIdentifier {
                        column: identifier_from_str("e"),
                        table: Some(identifier_from_str("d")),
                    }),
                    alias: Some(identifier_from_str("de")),
                },
            ],
            from: Some(SelectFrom {
                item: FromItem::Table(identifier_from_str("abc"), None),
                joins: vec![JoinItem {
                    item: FromItem::Table(
                        identifier_from_str("def"),
                        Some(identifier_from_str("d")),
                    ),
                    join: Join::Inner,
                    cond: Some(Expr::Operation(Operation::Equal(
                        Box::new(Expr::Column(ColumnIdentifier {
                            column: identifier_from_str("a"),
                            table: Some(identifier_from_str("abc")),
                        })),
                        Box::new(Expr::Column(ColumnIdentifier {
                            column: identifier_from_str("f"),
                            table: Some(identifier_from_str("d")),
                        })),
                    ))),
                }],
            }),
            cond: Some(Expr::Operation(Operation::Equal(
                Box::new(Expr::Column(ColumnIdentifier {
                    column: identifier_from_str("c"),
                    table: None,
                })),
                Box::new(Expr::Literal(Literal::Integer(0))),
            ))),
        }))];

        let output = Parser::new(input).parse();

        assert_eq!(output, expected_output);
    }
}
