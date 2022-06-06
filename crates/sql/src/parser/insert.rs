use crate::stmt::InsertSource;

use {
    super::{common::match_token, Parser},
    crate::{
        error::{Error, Result},
        lexer::{Keyword, Token},
        stmt::Stmt,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_insert(&mut self) -> Result<Stmt> {
        self.must_match(Token::Keyword(Keyword::INTO))?;

        let table = self.parse_identifier()?;
        let columns = match self.tokens.peek() {
            Some(Ok((Token::LeftParen, _))) => {
                Some(self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?)
            }
            _ => None,
        };

        let source = match_token!(self.tokens.next(), {
            (Token::Keyword(Keyword::VALUES), _) => {
                let values = self.parse_comma_separated_within_parentheses(Self::parse_expr, false)?;
                InsertSource::Values(values)
            },
            (Token::Keyword(Keyword::SELECT), _) => {
                let select = self.parse_select()?;
                InsertSource::FromSelect(Box::new(select))
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
    use crate::stmt::Expr;

    use {
        super::*,
        crate::{
            common::test_utils::identifier_from_str,
            stmt::{ColumnRef, FromItem, Literal, Select, SelectFrom, SelectItem, Stmt},
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
                source: InsertSource::Values(vec![
                    Expr::Literal(Literal::Integer(1)),
                    Expr::Literal(Literal::Float(3.14)),
                    Expr::Literal(Literal::Boolean(true)),
                ]),
            }),
            Ok(Stmt::Insert {
                table: identifier_from_str("def"),
                columns: None,
                source: InsertSource::FromSelect(Box::new(Select {
                    distinct: false,
                    targets: vec![
                        SelectItem {
                            expr: Expr::Column(ColumnRef {
                                column: identifier_from_str("a"),
                                table: None,
                            }),
                            alias: None,
                        },
                        SelectItem {
                            expr: Expr::Column(ColumnRef {
                                column: identifier_from_str("b"),
                                table: None,
                            }),
                            alias: None,
                        },
                    ],
                    from: Some(SelectFrom {
                        item: FromItem::Table(identifier_from_str("abc"), None),
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
