use {
    super::Parser,
    crate::{
        error::{Error, Result},
        lexer::{Keyword, Token},
        stmt::Stmt,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_create_database(&mut self) -> Result<Stmt> {
        let if_not_exists =
            self.match_keyword_sequence(&[Keyword::IF, Keyword::NOT, Keyword::EXISTS]);

        let name = match self.tokens.next() {
            Some(Ok((Token::Identifier, span))) => self.ident_from_span(span),

            Some(Ok((_, span))) => return Err(Error::SyntaxError(span)),
            Some(Err(e)) => return Err(e),
            None => return Err(Error::UnexpectedEnd),
        };

        Ok(Stmt::CreateDatabase {
            if_not_exists,
            name,
        })
    }
}
