use {
    super::{ast::Identifier, Parser},
    crate::{
        common::{DataType, Span, Spanned},
        error::{Error, Result},
        lexer::{Keyword, Token},
    },
    core::str::FromStr,
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

    pub(super) fn number_from_span<T: FromStr>(&self, span: Span) -> Result<T> {
        self.src[span.clone()]
            .parse::<T>()
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

    pub(super) fn parse_alias(&mut self) -> Result<Option<Identifier>> {
        if self.try_match(Token::Keyword(Keyword::AS)).is_some() {
            return Ok(Some(self.parse_identifier()?));
        }

        Ok(match self.try_match(Token::Identifier) {
            Some((_, span)) => Some(self.identifier_from_span(span)),
            None => None,
        })
    }

    pub(super) fn parse_comma_separated_within_parentheses<T, F>(
        &mut self,
        func: F,
        allow_empty: bool,
    ) -> Result<Spanned<Vec<T>>>
    where
        F: FnMut(&mut Parser<'a>) -> Result<T>,
    {
        let (_, s1) = self.must_match(Token::LeftParen)?;

        Ok(match self.tokens.peek() {
            Some(Ok((Token::RightParen, s2))) if allow_empty => {
                let end = *s2.end();
                self.tokens.next();
                (Vec::new(), (*s1.start()..=end))
            }
            _ => {
                let result = self.parse_comma_separated(func)?;
                let (_, s2) = self.must_match(Token::RightParen)?;
                (result, (*s1.start()..=*s2.end()))
            }
        })
    }

    pub(super) fn parse_comma_separated<T, F>(&mut self, mut func: F) -> Result<Vec<T>>
    where
        F: FnMut(&mut Parser<'a>) -> Result<T>,
    {
        let mut v = vec![];

        loop {
            v.push(func(self)?);

            if self.try_match(Token::Comma).is_none() {
                break;
            }
        }

        Ok(v)
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
                let (_, span) = self.must_match(Token::Number { is_float: false })?;
                self.must_match(Token::RightParen)?;

                let len = self.number_from_span(span)?;

                Ok(DataType::Char(len))
            },
            (Token::Keyword(Keyword::VARCHAR), _) => {
                self.must_match(Token::LeftParen)?;
                let (_, span) = self.must_match(Token::Number { is_float: false })?;
                self.must_match(Token::RightParen)?;

                let len = self.number_from_span(span)?;

                Ok(DataType::Varchar(len))
            },
        })
    }
}

macro_rules! match_token {
    ( $token:expr, { $( $($t:pat_param)|* $(if $cond:expr)? => $e:expr, )* } ) => {
        match $token {
            $( $( Some(Ok($t)) )|* $(if $cond)? => $e,)*

            Some(Ok((_, span))) => return Err(Error::SyntaxError(span)),
            Some(Err(e)) => return Err(e),
            None => return Err(Error::UnexpectedEnd),
        }
    };
}

pub(super) use match_token;
