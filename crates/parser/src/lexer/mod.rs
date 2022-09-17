mod error;
mod token;

pub(crate) use self::{
    error::{Error, Result},
    token::{Keyword, Token},
};

use {
    crate::common::Spanned,
    std::{
        iter::Peekable,
        str::{CharIndices, FromStr},
    },
};

pub(super) struct Lexer<'a> {
    src: &'a str,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Spanned<Token>>;

    fn next(&mut self) -> Option<Self::Item> {
        // consume whitespace
        self.iter_next_while(|c| c.is_whitespace());

        let token = match self.iter.peek() {
            Some((_, '\'')) => self.scan_string(),
            Some((_, c)) if c.is_digit(10) => self.scan_number().map(|item| Ok(item)),
            Some((_, c)) if c.is_alphabetic() => self.scan_identifier().map(|item| Ok(item)),
            Some(_) => self.scan_symbol().map(|item| Ok(item)),
            _ => None,
        };

        match token {
            None => self
                .iter
                .peek()
                .map(|&(i, c)| Err(Error::UnexpectedChar { c, location: i })),
            other => other,
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
        while self.iter.next_if(|(_, c)| func(c)).is_some() {}
    }

    fn iter_offset(&mut self) -> usize {
        match self.iter.peek() {
            Some(&(0, _)) => 0,
            Some(&(i, _)) => i - 1,
            None => self.src.len() - 1,
        }
    }

    fn scan_string(&mut self) -> Option<Result<Spanned<Token>>> {
        let begin = self.iter.next_if(|&(_, c)| c == '\'')?.0;

        while let Some((i, c)) = self.iter.next() {
            if c != '\'' {
                continue;
            }

            match self.iter.peek() {
                // check if it escapes a single quote
                Some((_, '\'')) => _ = self.iter.next(),
                _ => {
                    return Some(Ok(Spanned(Token::String, begin..=i)));
                }
            }
        }

        Some(Err(Error::NoClosingQuoteForString(
            begin..=self.src.len() - 1,
        )))
    }

    fn scan_number(&mut self) -> Option<Spanned<Token>> {
        let begin = self.iter.next_if(|&(_, c)| c.is_digit(10))?.0;

        self.iter_next_while(|c| c.is_digit(10));

        let is_float = self.iter.next_if(|&(_, c)| c == '.').is_some();

        self.iter_next_while(|c| c.is_digit(10));
        self.iter.next_if(|&(_, c)| c == 'e' || c == 'E');
        self.iter.next_if(|&(_, c)| c == '+' || c == '-');
        self.iter_next_while(|c| c.is_digit(10));

        Some(Spanned(
            Token::Number { is_float },
            begin..=self.iter_offset(),
        ))
    }

    fn scan_identifier(&mut self) -> Option<Spanned<Token>> {
        let begin = self.iter.next_if(|&(_, c)| c.is_alphabetic())?.0;

        self.iter_next_while(|&c| c.is_alphanumeric() || c == '_');

        let range = begin..=self.iter_offset();
        let ident = &self.src[range.clone()];

        let token = Keyword::from_str(ident)
            .map(Token::Keyword)
            .unwrap_or(Token::Identifier);

        Some(Spanned(token, range))
    }

    fn scan_symbol(&mut self) -> Option<Spanned<Token>> {
        let &(begin, next_char) = self.iter.peek()?;

        let mut iter_should_next = true;
        let symbol = match next_char {
            '.' => Some(Token::Period),
            '=' => Some(Token::Equal),
            '<' => {
                self.iter.next();
                match self.iter.peek() {
                    Some((_, '>')) => Some(Token::LessOrGreaterThan),
                    Some((_, '=')) => Some(Token::LessThanOrEqual),
                    _ => {
                        iter_should_next = false;
                        Some(Token::LessThan)
                    }
                }
            }
            '>' => {
                self.iter.next();
                match self.iter.peek() {
                    Some((_, '=')) => Some(Token::GreaterThanOrEqual),
                    _ => {
                        iter_should_next = false;
                        Some(Token::GreaterThan)
                    }
                }
            }
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Asterisk),
            '/' => Some(Token::Slash),
            '%' => Some(Token::Percent),
            '^' => Some(Token::Caret),
            '?' => Some(Token::Question),
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            ',' => Some(Token::Comma),
            ';' => Some(Token::Semicolon),
            '!' => {
                self.iter.next();
                self.iter
                    .peek()
                    .filter(|(_, c)| *c == '=')
                    .map(|_| Token::NotEqual)
            }
            _ => {
                iter_should_next = false;
                None
            }
        };

        if iter_should_next {
            self.iter.next();
        }

        symbol.map(|symbol| Spanned(symbol, begin..=self.iter_offset()))
    }
}

#[cfg(test)]
mod tests {
    use {super::*, std::iter::zip};

    fn test(input: &str, expected_output: &[Result<Spanned<Token>>]) {
        let lexer = Lexer::new(&input);
        let output = lexer.collect::<Vec<_>>();

        assert_eq!(output, expected_output);
    }

    fn construct_expected_output(
        input: &str,
        strs: Vec<&str>,
        tokens: Vec<Token>,
    ) -> Vec<Result<Spanned<Token>>> {
        assert_eq!(strs.len(), tokens.len());

        zip(strs, tokens)
            .into_iter()
            .map(|(s, token)| {
                let begin = input.find(s).unwrap();
                let range = begin..=begin + s.len() - 1;

                Ok(Spanned(token, range))
            })
            .collect::<Vec<_>>()
    }

    fn make_test(input: &str, tokens: Vec<Token>) {
        let strs = input.split_whitespace().collect();
        let expected_output = construct_expected_output(&input, strs, tokens);

        test(&input, &expected_output);
    }

    #[test]
    fn scan_string() {
        let input = " 'abc''DEF'  'ABC*DEF'  ";
        let tokens = vec![Token::String, Token::String];

        make_test(input, tokens);
    }

    #[test]
    fn scan_string_error() {
        let input = "'abc";
        let expected_output = [Err(Error::NoClosingQuoteForString(0..=3))];

        test(input, &expected_output);
    }

    #[test]
    fn scan_number() {
        let input = "12 123.  123.456e+789";
        let tokens = vec![
            Token::Number { is_float: false },
            Token::Number { is_float: true },
            Token::Number { is_float: true },
        ];

        make_test(input, tokens);
    }

    #[test]
    fn scan_identifier() {
        let input = " SELECT abc FROM def";
        let tokens = vec![
            Token::Keyword(Keyword::SELECT),
            Token::Identifier,
            Token::Keyword(Keyword::FROM),
            Token::Identifier,
        ];

        make_test(input, tokens);
    }

    #[test]
    fn scan_symbol() {
        let input = "* != < >= <>";
        let tokens = vec![
            Token::Asterisk,
            Token::NotEqual,
            Token::LessThan,
            Token::GreaterThanOrEqual,
            Token::LessOrGreaterThan,
        ];

        make_test(input, tokens);
    }
}
