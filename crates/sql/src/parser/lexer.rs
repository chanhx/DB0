use {
    super::token::Token,
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
        while let Some(_) = self.iter.next_if(|(_, c)| c.is_whitespace()) {}

        let token = match self.iter.peek() {
            Some((_, '\'')) => self.scan_string(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_string() {
        let input = "'abc''DEF'";
        let mut lexer = Lexer::new(input);

        match lexer.next() {
            Some(Ok(Token::String(s))) => assert_eq!(s, input),
            _ => unreachable!(),
        }
    }

    #[test]
    fn scan_string_error() {
        let input = "'abc";
        let mut lexer = Lexer::new(input);

        assert!(matches!(lexer.next(), Some(Err(_))));
    }
}
