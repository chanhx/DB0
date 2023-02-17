use {
    super::{
        common::match_token,
        error::{Error, Result},
        Parser,
    },
    ast::{
        token::{Keyword, Token},
        Column, ColumnConstraint, CreateTableAsStmt, CreateTableStmt, Spanned, Statement,
        TableConstraint, TableSchema,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_create(&mut self) -> Result<Statement> {
        // let or_replace = self.match_keyword_sequence(&[Keyword::OR, Keyword::REPLACE]);
        // let is_temp = self.match_keyword_aliases(&[Keyword::TEMP, Keyword::TEMPORARY]);

        match_token!(self.tokens.next(), {
            Spanned(Token::Keyword(Keyword::DATABASE), _) => self.parse_create_database(),
            Spanned(Token::Keyword(Keyword::TABLE), _) => self.parse_create_table(),
            Spanned(Token::Keyword(Keyword::INDEX), _) => self.parse_create_index(false),
            Spanned(Token::Keyword(Keyword::UNIQUE), _) => {
                self.must_match(Token::Keyword(Keyword::INDEX))?;
                self.parse_create_index(true)
            },
        })
    }

    pub(super) fn parse_drop(&mut self) -> Result<Statement> {
        match_token!(self.tokens.next(), {
            Spanned(Token::Keyword(Keyword::DATABASE), _) =>
                Ok(Statement::DropDatabase { name: self.parse_identifier()? }),
            Spanned(Token::Keyword(Keyword::TABLE), _) =>
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

        let (columns, constraints) = match self.tokens.peek() {
            Some(Ok(Spanned(Token::LeftParen, _))) => self.parse_table_schema()?,
            _ => (vec![], vec![]),
        };

        if self.try_match(Token::Keyword(Keyword::AS)).is_some() {
            let query = self.parse_select()?;

            let columns = if columns.len() > 0 {
                Some(columns)
            } else {
                None
            };

            Ok(Statement::CreateTableAs(CreateTableAsStmt {
                if_not_exists,
                name,
                columns,
                constraints,
                query,
            }))
        } else {
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

    fn parse_table_schema(&mut self) -> Result<(Vec<Column>, Vec<Spanned<TableConstraint>>)> {
        self.must_match(Token::LeftParen)?;

        let mut columns = vec![];
        let mut constraints = vec![];

        loop {
            match_token!(self.tokens.next(), {
                Spanned(Token::Identifier, span) => {
                    let name = self.identifier_from_span(span);
                    let data_type = self.parse_data_type()?;
                    let constraints = self.parse_column_constraint()?;

                    columns.push(Column {
                        name,
                        data_type,
                        constraints,
                    });
                },
                Spanned(Token::Keyword(Keyword::PRIMARY), s1) => {
                    self.must_match(Token::Keyword(Keyword::KEY))?;
                    let Spanned(columns, s2) = self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;

                    constraints.push(Spanned(TableConstraint::PrimaryKey(columns), *s1.start()..=*s2.end()));
                },
                Spanned(Token::Keyword(Keyword::UNIQUE), s1) => {
                    let Spanned(columns, s2) = self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;

                    constraints.push(Spanned(
                        TableConstraint::Unique(columns),
                        *s1.start()..=*s2.end(),
                    ));
                },
            });

            match_token!(self.tokens.next(), {
                Spanned(Token::Comma, _) => {},
                Spanned(Token::RightParen, _) => break,
            })
        }

        Ok((columns, constraints))
    }

    fn parse_column_constraint(&mut self) -> Result<Vec<Spanned<ColumnConstraint>>> {
        let mut constraints = Vec::new();

        while let Some(Ok(Spanned(Token::Keyword(keyword), s1))) = self
            .tokens
            .next_if(|item| matches!(item, Ok(Spanned(Token::Keyword(_), _))))
        {
            let Spanned(constraint, span) = match keyword {
                Keyword::PRIMARY => {
                    let Spanned(_, s2) = self.must_match(Token::Keyword(Keyword::KEY))?;
                    Spanned(ColumnConstraint::PrimaryKey, *s1.start()..=*s2.end())
                }
                Keyword::UNIQUE => Spanned(ColumnConstraint::Unique, s1),
                Keyword::NOT => {
                    let Spanned(_, s2) = self.must_match(Token::Keyword(Keyword::NULL))?;
                    Spanned(ColumnConstraint::NotNull, *s1.start()..=*s2.end())
                }
                _ => return Err(Error::SyntaxError(s1)),
            };

            constraints.push(Spanned(constraint, span));
        }

        Ok(constraints)
    }

    fn parse_create_index(&mut self, is_unique: bool) -> Result<Statement> {
        let name = self.parse_identifier()?;

        self.must_match(Token::Keyword(Keyword::ON))?;

        let table = self.parse_identifier()?;
        let Spanned(columns, _) =
            self.parse_comma_separated_within_parentheses(Self::parse_identifier, false)?;

        Ok(Statement::CreateIndex {
            is_unique,
            name,
            table,
            columns,
        })
    }
}
