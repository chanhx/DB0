use {
    super::token::{Keyword, Token},
    crate::error::{Details, Error, Result},
    std::{iter::Peekable, str::CharIndices},
};

pub(super) struct Lexer<'a> {
    src: &'a str,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        // consume whitespace
        self.iter_next_while(|c| c.is_whitespace());

        let token = match self.iter.peek() {
            Some((_, '\'')) => self.scan_string(),
            Some((_, c)) if c.is_digit(10) => Ok(self.scan_number()),
            Some((_, c)) if c.is_alphabetic() => Ok(self.scan_identifier()),
            _ => Ok(None),
        };

        match token {
            Ok(Some(token)) => Some(Ok(token)),
            Ok(None) => self
                .iter
                .peek()
                .map(|&(i, c)| Err(Error::new(self.src, i, Details::UnexpectedChar(c)))),
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> Lexer<'a> {
    pub(super) fn new(src: &'a str) -> Self {
        Self {
            src,
            iter: src.char_indices().peekable(),
        }
    }

    fn iter_next_while(&mut self, func: impl Fn(&char) -> bool) {
        while let Some(_) = self.iter.next_if(|(_, c)| func(c)) {}
    }

    fn iter_offset(&mut self) -> usize {
        self.iter
            .peek()
            .map(|&(i, _)| i - 1)
            .unwrap_or(self.src.len() - 1)
    }

    fn scan_string(&mut self) -> Result<Option<Token<'a>>> {
        let begin = self.iter.next_if(|&(_, c)| c == '\'');
        if begin.is_none() {
            return Ok(None);
        }

        let begin = begin.unwrap().0;

        while let Some((i, c)) = self.iter.next() {
            match c {
                '\'' => match self.iter.peek() {
                    // check if it escapes a single quote
                    Some((_, '\'')) => _ = self.iter.next(),
                    _ => {
                        // TODO: deal with escaping
                        return Ok(Some(Token::String(&self.src[begin..=i])));
                    }
                },
                _ => {}
            }
        }

        Err(Error::new(
            self.src,
            self.src.len() - 1,
            Details::NoClosingQuoteForString,
        ))
    }

    fn scan_number(&mut self) -> Option<Token<'a>> {
        let begin = self.iter.next_if(|&(_, c)| c.is_digit(10));
        if begin.is_none() {
            return None;
        }

        let begin = begin.unwrap().0;

        self.iter_next_while(|c| c.is_digit(10));
        self.iter.next_if(|&(_, c)| c == '.');
        self.iter_next_while(|c| c.is_digit(10));
        self.iter.next_if(|&(_, c)| c == 'e' || c == 'E');
        self.iter.next_if(|&(_, c)| c == '+' || c == '-');
        self.iter_next_while(|c| c.is_digit(10));

        let end = self.iter_offset();

        Some(Token::Number(&self.src[begin..=end]))
    }

    fn scan_identifier(&mut self) -> Option<Token<'a>> {
        let begin = self.iter.next_if(|&(_, c)| c.is_alphabetic());
        if begin.is_none() {
            return None;
        }

        let begin = begin.unwrap().0;
        self.iter_next_while(|&c| c.is_alphanumeric() || c == '_');
        let end = self.iter_offset();

        let ident = &self.src[begin..=end];

        Keyword::from_str(ident)
            .map(Token::Keyword)
            .or_else(|| Some(Token::Identifier(ident)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_string() {
        let strs = ["'abc''DEF'", "'ABC*DEF'"];
        let input = strs.join(" ");
        let expected_output = strs
            .iter()
            .map(|&s| Some(Ok(Token::String(s))))
            .collect::<Vec<_>>();
        let mut lexer = Lexer::new(&input);

        expected_output
            .iter()
            .for_each(|output| assert_eq!(*output, lexer.next()));
    }

    #[test]
    fn scan_string_error() {
        let input = "'abc";
        let mut lexer = Lexer::new(input);

        assert!(matches!(lexer.next(), Some(Err(_))));
    }

    #[test]
    fn scan_number() {
        let nums = ["123.", "123.456e+789"];
        let input = nums.join(" ");
        let expected_output = nums
            .iter()
            .map(|&s| Some(Ok(Token::Number(s))))
            .collect::<Vec<_>>();

        let mut lexer = Lexer::new(&input);

        expected_output
            .iter()
            .for_each(|output| assert_eq!(*output, lexer.next()));
    }

    #[test]
    fn scan_ident() {
        let input = "SELECT abc FROM def";
        let expected_output = [
            Some(Ok(Token::Keyword(Keyword::Select))),
            Some(Ok(Token::Identifier("abc"))),
            Some(Ok(Token::Keyword(Keyword::From))),
            Some(Ok(Token::Identifier("def"))),
        ];

        let mut lexer = Lexer::new(&input);

        expected_output
            .iter()
            .for_each(|token| assert_eq!(*token, lexer.next()));
    }
}
