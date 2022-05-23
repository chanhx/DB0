mod common;
mod ddl;

use {
    crate::{
        common::iter::{MultiPeek, MultiPeekable},
        error::{Error, Result},
        lexer::{Keyword, Lexer, Token},
        stmt::Stmt,
    },
    common::match_token,
};

pub struct Parser<'a> {
    src: &'a str,
    tokens: MultiPeekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            tokens: Lexer::new(src).multi_peekable(),
        }
    }

    pub fn parse(mut self) -> Vec<Result<Stmt>> {
        let mut stmts = Vec::new();
        while let Some(stmt) = self.parse_statement() {
            let met_err = stmt.is_err();
            stmts.push(stmt);

            if met_err {
                break;
            }
        }

        stmts
    }

    fn parse_statement(&mut self) -> Option<Result<Stmt>> {
        self.skip_semicolons();

        Some(match self.tokens.next()? {
            Ok((Token::Keyword(Keyword::CREATE), _)) => self.parse_create(),
            Ok((_, span)) => Err(Error::SyntaxError(span)),
            Err(e) => Err(e),
        })
    }

    fn parse_create(&mut self) -> Result<Stmt> {
        // let or_replace = self.match_keyword_sequence(&[Keyword::OR, Keyword::REPLACE]);
        // let is_temp = self.match_keyword_aliases(&[Keyword::TEMP, Keyword::TEMPORARY]);

        match_token!(self.tokens.next(), {
            (Token::Keyword(Keyword::DATABASE), _) => self.parse_create_database(),
            (Token::Keyword(Keyword::TABLE), _) => self.parse_create_table(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::{Error, Result},
        stmt::{Column, ColumnConstraint, DataType, Identifier, Stmt, TableConstraint},
    };

    fn identifier_from_str(s: &str) -> Identifier {
        Identifier(s.to_string(), 0..=s.len() - 1)
    }

    #[test]
    fn parse_ddl() {
        let sql = "
            CReaTe Database if NOT Exists abc;

            CREATE TABLE if NOT Exists abc (
                PRIMARY KEY (a),
                a int,
                b varchar(15) Not null,
                c integer unique
            );
        ";

        let expected_output: Vec<Result<_>> = vec![
            Ok(Stmt::CreateDatabase {
                if_not_exists: true,
                name: identifier_from_str("abc"),
            }),
            Ok(Stmt::CreateTable {
                if_not_exists: true,
                name: identifier_from_str("abc"),
                columns: vec![
                    Column {
                        name: identifier_from_str("a"),
                        data_type: DataType::Integer,
                        constraints: vec![],
                    },
                    Column {
                        name: identifier_from_str("b"),
                        data_type: DataType::Varchar(15),
                        constraints: vec![ColumnConstraint::NotNull],
                    },
                    Column {
                        name: identifier_from_str("c"),
                        data_type: DataType::Integer,
                        constraints: vec![ColumnConstraint::Unique],
                    },
                ],
                constraints: vec![TableConstraint::PrimaryKey(vec![identifier_from_str("a")])],
                from_query: None,
            }),
        ];

        let output = Parser::new(sql).parse();

        assert_eq!(output.len(), expected_output.len());
        std::iter::zip(output, expected_output).for_each(|(a, b)| {
            assert_eq!(a, b);
        });
    }
}
