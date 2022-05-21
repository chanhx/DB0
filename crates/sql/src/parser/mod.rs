mod common;
mod ddl;

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

        let (keyword, span) = match self.tokens.next()? {
            Ok(item) => item,
            Err(e) => return Some(Err(e)),
        };

        Some(match keyword {
            Token::Keyword(keyword) => match keyword {
                Keyword::CREATE => self.parse_create(),
                _ => Err(Error::SyntaxError(span)),
            },
            // Token::LeftParen => {}
            _ => Err(Error::SyntaxError(span)),
        })
    }

    fn parse_create(&mut self) -> Result<Stmt> {
        // let or_replace = self.match_keyword_sequence(&[Keyword::OR, Keyword::REPLACE]);
        // let is_temp = self.match_keyword_aliases(&[Keyword::TEMP, Keyword::TEMPORARY]);

        match self.tokens.next() {
            Some(Ok((Token::Keyword(Keyword::DATABASE), _))) => self.parse_create_database(),

            Some(Ok((_, span))) => Err(Error::SyntaxError(span)),
            Some(Err(e)) => Err(e),
            None => Err(Error::UnexpectedEnd),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let sql = "CReaTe Database if NOT Exists abc;";
        let expected_output: Vec<Result<_>> = vec![Ok(Stmt::CreateDatabase {
            name: "abc".to_string(),
            if_not_exists: true,
        })];

        let output = Parser::new(sql).parse();

        assert_eq!(output.len(), expected_output.len());
        std::iter::zip(output, expected_output).for_each(|(a, b)| {
            assert_eq!(a, b);
        });
    }
}
