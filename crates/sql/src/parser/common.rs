use {
    super::Parser,
    crate::{
        common::Span,
        error::{Error, Result},
        lexer::{Keyword, Token},
        stmt::{DataType, Identifier},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn identifier_from_span(&self, span: Span) -> Identifier {
        Identifier(self.src[span.clone()].to_string(), span.clone())
    }

    pub(super) fn string_from_span(&self, span: Span) -> String {
        // Modify the start and end of the span to trim the opening and closing single quotes,
        // and then escape single quotes.
        let (start, end) = (span.start() + 1, span.end() - 1);
        self.src[start..=end].to_string().replace("''", "'")
    }

    pub(super) fn integer_from_span(&self, span: Span) -> Result<isize> {
        self.src[span.clone()]
            .parse()
            .map_err(|_| Error::SyntaxError(span.clone()))
    }

    pub(super) fn skip_semicolons(&mut self) {
        while self
            .tokens
            .next_if(|token| matches!(token, Ok((Token::Semicolon, _))))
            .is_some()
        {}
    }

    pub(super) fn match_keyword_sequence(&mut self, keywords: &[Keyword]) -> bool {
        self.tokens
            .advance_n_if_each(keywords.len(), |(i, token)| match token {
                Ok((Token::Keyword(keyword), _)) => *keyword == keywords[i],
                _ => false,
            })
            .is_some()
    }

    pub(super) fn must_match(&mut self, token: Token) -> Result<(Token, Span)> {
        match_token!(self.tokens.next(), {
            (t, span) if t == token => {
                Ok((t, span))
            },
        })
    }

    pub(super) fn try_match(&mut self, token: Token) -> Option<(Token, Span)> {
        self.tokens
            .next_if(|item| match item {
                Ok((t, _)) => *t == token,
                _ => false,
            })
            .map(|item| item.unwrap())
    }

    pub(super) fn parse_identifier(&mut self) -> Result<Identifier> {
        match_token!(self.tokens.next(), {
            (Token::Identifier, span) => Ok(self.identifier_from_span(span)),
        })
    }

    pub(super) fn parse_identifiers_within_parentheses(&mut self) -> Result<Vec<Identifier>> {
        let mut identifiers = Vec::new();

        self.must_match(Token::LeftParen)?;
        loop {
            let identifier = self.parse_identifier()?;
            identifiers.push(identifier);

            match_token!(self.tokens.next(), {
                (Token::Comma, _) => {},
                (Token::RightParen, _) => break,
            });
        }

        Ok(identifiers)
    }

    pub(super) fn parse_data_type(&mut self) -> Result<DataType> {
        match_token!(self.tokens.next(), {
            (Token::Keyword(Keyword::BOOLEAN), _) => Ok(DataType::Boolean),
            (Token::Keyword(Keyword::SMALLINT), _) => Ok(DataType::SmallInt),
            (Token::Keyword(Keyword::INTEGER), _)
            | (Token::Keyword(Keyword::INT), _) => Ok(DataType::Integer),
            (Token::Keyword(Keyword::DECIMAL), _)
            | (Token::Keyword(Keyword::NUMERIC), _) => Ok(DataType::Decimal),
            (Token::Keyword(Keyword::FLOAT), _) => Ok(DataType::Float),
            (Token::Keyword(Keyword::CHAR), _) => {
                self.must_match(Token::LeftParen)?;
                let (_, span) = self.must_match(Token::Number)?;
                self.must_match(Token::RightParen)?;

                let len = self.integer_from_span(span)?;

                Ok(DataType::Char(len as u32))
            },
            (Token::Keyword(Keyword::VARCHAR), _) => {
                self.must_match(Token::LeftParen)?;
                let (_, span) = self.must_match(Token::Number)?;
                self.must_match(Token::RightParen)?;

                let len = self.integer_from_span(span)?;

                Ok(DataType::Varchar(len as u32))
            },
        })
    }
}

macro_rules! match_token {
    ( $token:expr, { $($t:tt $(| $ot:tt)* $(if $cond:expr)? => $e:expr,)* } ) => {
        match $token {
            $(Some(Ok($t)) $(| Some(Ok($ot)))* $(if $cond)? => $e,)*

            Some(Ok((_, span))) => return Err(Error::SyntaxError(span)),
            Some(Err(e)) => return Err(e),
            None => return Err(Error::UnexpectedEnd),
        }
    };
}

pub(super) use match_token;
