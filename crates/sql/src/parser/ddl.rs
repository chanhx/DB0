use {
    super::{common::match_token, Parser},
    crate::{
        error::{Error, Result},
        lexer::{Keyword, Token},
        stmt::{Column, ColumnConstraint, Stmt, TableConstraint},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_create(&mut self) -> Result<Stmt> {
        // let or_replace = self.match_keyword_sequence(&[Keyword::OR, Keyword::REPLACE]);
        // let is_temp = self.match_keyword_aliases(&[Keyword::TEMP, Keyword::TEMPORARY]);

        match_token!(self.tokens.next(), {
            (Token::Keyword(Keyword::DATABASE), _) => self.parse_create_database(),
            (Token::Keyword(Keyword::TABLE), _) => self.parse_create_table(),
            (Token::Keyword(Keyword::INDEX), _) => self.parse_create_index(false),
            (Token::Keyword(Keyword::UNIQUE), _) => {
                self.must_match(Token::Keyword(Keyword::INDEX))?;
                self.parse_create_index(true)
            },
        })
    }

    pub(super) fn parse_drop(&mut self) -> Result<Stmt> {
        match_token!(self.tokens.next(), {
            (Token::Keyword(Keyword::DATABASE), _) =>
                Ok(Stmt::DropDatabase { name: self.parse_identifier()? }),
            (Token::Keyword(Keyword::TABLE), _) =>
                Ok(Stmt::DropTable { name: self.parse_identifier()? }),
        })
    }

    pub(super) fn parse_create_database(&mut self) -> Result<Stmt> {
        let if_not_exists =
            self.match_keyword_sequence(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = self.parse_identifier()?;

        Ok(Stmt::CreateDatabase {
            if_not_exists,
            name,
        })
    }

    pub(super) fn parse_create_table(&mut self) -> Result<Stmt> {
        let if_not_exists =
            self.match_keyword_sequence(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = self.parse_identifier()?;
        let (columns, constraints) = self.parse_table_structure()?;

        Ok(Stmt::CreateTable {
            if_not_exists,
            name,
            columns,
            constraints,
            from_query: None,
        })
    }

    pub(super) fn parse_table_structure(&mut self) -> Result<(Vec<Column>, Vec<TableConstraint>)> {
        if self.try_match(Token::LeftParen).is_none() {
            return Ok((vec![], vec![]));
        }

        if self.try_match(Token::RightParen).is_some() {
            return Ok((vec![], vec![]));
        }

        let mut columns = vec![];
        let mut table_constraints = vec![];

        loop {
            match_token!(self.tokens.next(), {
                (Token::Identifier, span) => {
                    let name = self.identifier_from_span(span);
                    let data_type = self.parse_data_type()?;
                    let constraints = self.parse_column_constraint()?;

                    columns.push(Column {
                        name,
                        data_type,
                        constraints,
                    });
                },
                (Token::Keyword(Keyword::PRIMARY), _) => {
                    self.must_match(Token::Keyword(Keyword::KEY))?;
                    let columns = self.parse_identifiers_within_parentheses()?;

                    table_constraints.push(TableConstraint::PrimaryKey(columns));
                },
                (Token::Keyword(Keyword::UNIQUE), _) => {
                    let name = self.try_match(Token::Identifier).map(|item| {
                        self.identifier_from_span(item.1)
                    });
                    let columns = self.parse_identifiers_within_parentheses()?;

                    table_constraints.push(TableConstraint::Unique {
                        name,
                        columns,
                    });
                },
            });

            match_token!(self.tokens.next(), {
                (Token::Comma, _) => {},
                (Token::RightParen, _) => break,
            })
        }

        Ok((columns, table_constraints))
    }

    pub(super) fn parse_column_constraint(&mut self) -> Result<Vec<ColumnConstraint>> {
        let mut constraints = Vec::new();

        while let Some(Ok((Token::Keyword(keyword), span))) = self
            .tokens
            .next_if(|item| matches!(item, Ok((Token::Keyword(_), _))))
        {
            constraints.push(match keyword {
                Keyword::PRIMARY => {
                    self.must_match(Token::Keyword(Keyword::KEY))?;
                    ColumnConstraint::PrimaryKey
                }
                Keyword::UNIQUE => ColumnConstraint::Unique,
                Keyword::NOT => {
                    self.must_match(Token::Keyword(Keyword::NULL))?;
                    ColumnConstraint::NotNull
                }
                _ => return Err(Error::SyntaxError(span)),
            });
        }

        Ok(constraints)
    }

    pub(super) fn parse_create_index(&mut self, is_unique: bool) -> Result<Stmt> {
        let name = self.parse_identifier()?;

        self.must_match(Token::Keyword(Keyword::ON))?;

        let table = self.parse_identifier()?;
        let columns = self.parse_identifiers_within_parentheses()?;

        Ok(Stmt::CreateIndex {
            is_unique,
            name,
            table,
            columns,
        })
    }
}
