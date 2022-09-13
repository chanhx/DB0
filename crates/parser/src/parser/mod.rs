mod common;
mod ddl;
mod error;
mod expr;
mod insert;
mod query;

pub use self::error::{Error, Result};

use crate::{
    ast::Statement,
    common::{MultiPeek, MultiPeekable},
    lexer::{Keyword, Lexer, Token},
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

    pub fn parse(sql: &'a str) -> Vec<Result<Statement>> {
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

    fn parse_statement(&mut self) -> Option<Result<Statement>> {
        self.skip_semicolons();

        Some(match self.tokens.next()? {
            Ok((Token::Keyword(Keyword::CREATE), _)) => self.parse_create(),
            Ok((Token::Keyword(Keyword::DROP), _)) => self.parse_drop(),
            Ok((Token::Keyword(Keyword::INSERT), _)) => self.parse_insert(),
            Ok((Token::Keyword(Keyword::SELECT), _)) => {
                self.parse_select().map(|select| Statement::Select(select))
            }
            Ok((_, span)) => Err(Error::SyntaxError(span)),
            Err(e) => Err(Error::LexingError(e)),
        })
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::ast::{
            identifier_from_str, Column, ColumnConstraint, CreateTableStmt, Statement,
            TableConstraint, TableSchema,
        },
        def::DataType,
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
            Ok(Statement::CreateDatabase {
                if_not_exists: true,
                name: identifier_from_str("abc"),
            }),
            Ok(Statement::CreateTable(CreateTableStmt {
                if_not_exists: true,
                name: identifier_from_str("abc"),
                table_schema: TableSchema {
                    columns: vec![
                        Column {
                            name: identifier_from_str("a"),
                            data_type: DataType::Integer,
                            constraints: vec![],
                        },
                        Column {
                            name: identifier_from_str("b"),
                            data_type: DataType::Varchar(15),
                            constraints: vec![(ColumnConstraint::NotNull, (180..=187))],
                        },
                        Column {
                            name: identifier_from_str("c"),
                            data_type: DataType::Integer,
                            constraints: vec![(ColumnConstraint::Unique, (216..=221))],
                        },
                    ],
                    constraints: vec![(
                        TableConstraint::PrimaryKey(vec![identifier_from_str("a")]),
                        (110..=124),
                    )],
                },
            })),
            Ok(Statement::DropDatabase {
                name: identifier_from_str("abc"),
            }),
            Ok(Statement::DropTable {
                name: identifier_from_str("a123"),
            }),
            Ok(Statement::CreateIndex {
                is_unique: false,
                name: identifier_from_str("hi"),
                table: identifier_from_str("abc"),
                columns: vec![identifier_from_str("a"), identifier_from_str("b")],
            }),
            Ok(Statement::CreateIndex {
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
