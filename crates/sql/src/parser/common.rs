use {
    super::Parser,
    crate::{
        common::Span,
        lexer::{Keyword, Token},
    },
};

impl<'a> Parser<'a> {
    pub(super) fn ident_from_span(&self, span: Span) -> String {
        self.src[span].to_string()
    }

    pub(super) fn string_from_span(&self, span: Span) -> String {
        // Modify the start and end of the span to trim the opening and closing single quotes,
        // and then escape single quotes.
        let (start, end) = (span.start() + 1, span.end() - 1);
        self.src[start..=end].to_string().replace("''", "'")
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

    pub(super) fn match_keyword_aliases(&mut self, aliases: &[Keyword]) -> bool {
        self.tokens
            .next_if(|item| match item {
                Ok((Token::Keyword(keyword), _)) => aliases.iter().any(|alias| *alias == *keyword),
                _ => false,
            })
            .is_some()
    }
}
