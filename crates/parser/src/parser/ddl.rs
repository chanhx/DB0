use {
    super::{
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    crate::{
        ast::{
            Column, ColumnConstraint, CreateTableAsStmt, CreateTableStmt, Identifier, Statement,
            TableConstraint, TableSchema,
        },
        common::Spanned,
        lexer::{Keyword, Token},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_create(&mut self) -> Result<Statement> {
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

    pub(super) fn parse_drop(&mut self) -> Result<Statement> {
        match_token!(self.tokens.next(), {
            (Token::Keyword(Keyword::DATABASE), _) =>
                Ok(Statement::DropDatabase { name: self.parse_identifier()? }),
            (Token::Keyword(Keyword::TABLE), _) =>
                Ok(Statement::DropTable { name: self.parse_identifier()? }),
        })
    }

    fn parse_create_database(&mut self) -> Result<Statement> {
        let if_not_exists =
            self.match_keyword_sequence(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = self.parse_identifier()?;

        Ok(Statement::CreateDatabase {
            if_not_exists,
            name,
        })
    }

    fn parse_create_table(&mut self) -> Result<Statement> {
        let if_not_exists =
            self.match_keyword_sequence(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = self.parse_identifier()?;

        let mut n = 0;
        let mut unmatched_bracket = 0;
        for i in 0.. {
            match self.tokens.peek_nth(i) {
                Some(Ok((Token::LeftParen, _))) => unmatched_bracket += 1,
                Some(Ok((Token::RightParen, range))) => {
                    if unmatched_bracket == 0 {
                        return Err(Error::SyntaxError(range.clone()));
                    }
                    unmatched_bracket -= 1;
                }
                Some(Err(e)) => return Err(Error::LexingError(e.clone())),
                Some(_) => {}
                None => return Err(Error::UnexpectedEnd),
            }

            if unmatched_bracket == 0 {
                n = i + 1;
                break;
            }
        }

        match self.tokens.peek_nth(n) {
            Some(Ok((Token::Keyword(Keyword::AS), _))) => {
                let (columns, constraints) = match self.tokens.peek() {
                    Some(Ok((Token::LeftParen, _))) => {
                        let (columns, _, constraints) = self.parse_table_schema(true)?;
                        (Some(columns), constraints)
                    }
                    _ => (None, vec![]),
                };
                let query = self.parse_select()?;

                Ok(Statement::CreateTableAs(CreateTableAsStmt {
                    if_not_exists,
                    name,
                    columns,
                    constraints,
                    query,
                }))
            }
            _ => {
                let (_, columns, constraints) = self.parse_table_schema(false)?;
                let table_schema = TableSchema {
                    columns,
                    constraints,
                };

                Ok(Statement::CreateTable(CreateTableStmt {
                    if_not_exists,
                    name,
                    table_schema,
                }))
            }
        }
    }

    fn parse_table_schema(
        &mut self,
        from_query: bool,
    ) -> Result<(Vec<Identifier>, Vec<Column>, Vec<Spanned<TableConstraint>>)> {
        self.must_match(Token::LeftParen)?;

        let mut column_names = vec![];
        let mut column_defs = vec![];
        let mut constraints = vec![];

        loop {
            match_token!(self.tokens.next(), {
                (Token::Identifier, span) if from_query => {
                    let name = self.identifier_from_span(span);

                    column_names.push(name);
                },

                (Token::Identifier, span) => {
                    let name = self.identifier_from_span(span);
                    let data_type = self.parse_data_type()?;
                    let constraints = self.parse_column_constraint()?;

                    column_defs.push(Column {
                        name,
                        data_type,
                        constraints,
                    });
                },
                (Token::Keyword(Keyword::PRIMARY), s1) => {
                    self.must_match(Token::Keyword(Keyword::KEY))?;
                    let (columns, s2) = self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;

                    constraints.push((TableConstraint::PrimaryKey(columns), (*s1.start()..=*s2.end())));
                },
                (Token::Keyword(Keyword::UNIQUE), s1) => {
                    let (columns, s2) = self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;

                    constraints.push((
                        TableConstraint::Unique(columns),
                        (*s1.start()..=*s2.end()),
                    ));
                },
            });

            match_token!(self.tokens.next(), {
                (Token::Comma, _) => {},
                (Token::RightParen, _) => break,
            })
        }

        Ok((column_names, column_defs, constraints))
    }

    fn parse_column_constraint(&mut self) -> Result<Vec<Spanned<ColumnConstraint>>> {
        let mut constraints = Vec::new();

        while let Some(Ok((Token::Keyword(keyword), s1))) = self
            .tokens
            .next_if(|item| matches!(item, Ok((Token::Keyword(_), _))))
        {
            let (constraint, span) = match keyword {
                Keyword::PRIMARY => {
                    let (_, s2) = self.must_match(Token::Keyword(Keyword::KEY))?;
                    (ColumnConstraint::PrimaryKey, (*s1.start()..=*s2.end()))
                }
                Keyword::UNIQUE => (ColumnConstraint::Unique, s1),
                Keyword::NOT => {
                    let (_, s2) = self.must_match(Token::Keyword(Keyword::NULL))?;
                    (ColumnConstraint::NotNull, (*s1.start()..=*s2.end()))
                }
                _ => return Err(Error::SyntaxError(s1)),
            };

            constraints.push((constraint, span));
        }

        Ok(constraints)
    }

    fn parse_create_index(&mut self, is_unique: bool) -> Result<Statement> {
        let name = self.parse_identifier()?;

        self.must_match(Token::Keyword(Keyword::ON))?;

        let table = self.parse_identifier()?;
        let (columns, _) =
            self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;

        Ok(Statement::CreateIndex {
            is_unique,
            name,
            table,
            columns,
        })
    }
}
