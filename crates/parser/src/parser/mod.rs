mod common;
mod ddl;
mod error;
mod expr;
mod insert;
mod query;

pub use self::error::{Error, Result};

use crate::{
    ast::Statement,
    common::{MultiPeek, MultiPeekable, Spanned},
    lexer::{Keyword, Lexer, Token},
};

pub struct Parser<'a> {
    src: &'a str,
    tokens: MultiPeekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(src: &'a str) -> Self {
        Self {
            src,
            tokens: Lexer::new(src).multi_peekable(),
        }
    }

    pub fn parse(sql: &'a str) -> Result<Vec<Statement>> {
        Self::new(sql).into_iter().collect()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Statement>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_semicolons();

        Some(match self.tokens.next()? {
            Ok(Spanned(Token::Keyword(Keyword::CREATE), _)) => self.parse_create(),
            Ok(Spanned(Token::Keyword(Keyword::DROP), _)) => self.parse_drop(),
            Ok(Spanned(Token::Keyword(Keyword::INSERT), _)) => self.parse_insert(),
            Ok(Spanned(Token::Keyword(Keyword::SELECT), _)) => {
                self.parse_select().map(|select| Statement::Select(select))
            }
            Ok(Spanned(_, span)) => Err(Error::SyntaxError(span)),
            Err(e) => Err(Error::LexingError(e)),
        })
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{ast::ddl::*, common::identifier_from_str},
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

        let expected_output = vec![
            Statement::CreateDatabase {
                if_not_exists: true,
                name: identifier_from_str("abc"),
            },
            Statement::CreateTable(CreateTableStmt {
                if_not_exists: true,
                name: identifier_from_str("abc"),
                table_schema: TableSchema {
                    columns: vec![
                        Column {
                            name: identifier_from_str("a"),
                            data_type: DataType::Int,
                            constraints: vec![],
                        },
                        Column {
                            name: identifier_from_str("b"),
                            data_type: DataType::Varchar(15),
                            constraints: vec![Spanned(ColumnConstraint::NotNull, 180..=187)],
                        },
                        Column {
                            name: identifier_from_str("c"),
                            data_type: DataType::Int,
                            constraints: vec![Spanned(ColumnConstraint::Unique, 216..=221)],
                        },
                    ],
                    constraints: vec![Spanned(
                        TableConstraint::PrimaryKey(vec![identifier_from_str("a")]),
                        110..=124,
                    )],
                },
            }),
            Statement::DropDatabase {
                name: identifier_from_str("abc"),
            },
            Statement::DropTable {
                name: identifier_from_str("a123"),
            },
            Statement::CreateIndex {
                is_unique: false,
                name: identifier_from_str("hi"),
                table: identifier_from_str("abc"),
                columns: vec![identifier_from_str("a"), identifier_from_str("b")],
            },
            Statement::CreateIndex {
                is_unique: true,
                name: identifier_from_str("hello"),
                table: identifier_from_str("abc"),
                columns: vec![identifier_from_str("a")],
            },
        ];

        let output = Parser::parse(sql).unwrap();

        assert_eq!(output.len(), expected_output.len());
        std::iter::zip(output, expected_output).for_each(|(a, b)| {
            assert_eq!(a, b);
        });
    }
}
