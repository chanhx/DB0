mod common;
mod ddl;
mod expr;
mod insert;
mod query;

use crate::{
    common::iter::{MultiPeek, MultiPeekable},
    error::{Error, Result},
    lexer::{Keyword, Lexer, Token},
    stmt::Stmt,
};

pub struct Parser<'a> {
    src: &'a str,
    tokens: MultiPeekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            tokens: Lexer::new(src).multi_peekable(),
        }
    }

    pub fn parse(sql: &'a str) -> Vec<Result<Stmt>> {
        let mut parser = Self::new(sql);

        let mut stmts = Vec::new();
        while let Some(stmt) = parser.parse_statement() {
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
            Ok((Token::Keyword(Keyword::DROP), _)) => self.parse_drop(),
            Ok((Token::Keyword(Keyword::INSERT), _)) => self.parse_insert(),
            Ok((Token::Keyword(Keyword::SELECT), _)) => {
                self.parse_select().map(|select| Stmt::Select(select))
            }
            Ok((_, span)) => Err(Error::SyntaxError(span)),
            Err(e) => Err(e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::test_utils::identifier_from_str,
        error::Result,
        stmt::{Column, ColumnConstraint, DataType, Stmt, TableConstraint},
    };

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

            DROP DATABASE abc;

            DROP TABLE a123;

            CREATE INDEX hi on abc (a, b);
            CREATE unique INDEX hello on abc (a);
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
            Ok(Stmt::DropDatabase {
                name: identifier_from_str("abc"),
            }),
            Ok(Stmt::DropTable {
                name: identifier_from_str("a123"),
            }),
            Ok(Stmt::CreateIndex {
                is_unique: false,
                name: identifier_from_str("hi"),
                table: identifier_from_str("abc"),
                columns: vec![identifier_from_str("a"), identifier_from_str("b")],
            }),
            Ok(Stmt::CreateIndex {
                is_unique: true,
                name: identifier_from_str("hello"),
                table: identifier_from_str("abc"),
                columns: vec![identifier_from_str("a")],
            }),
        ];

        let output = Parser::parse(sql);

        assert_eq!(output.len(), expected_output.len());
        std::iter::zip(output, expected_output).for_each(|(a, b)| {
            assert_eq!(a, b);
        });
    }
}
