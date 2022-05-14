use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    line: usize,
    column: usize,
    details: Details,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} [Ln {}, Col {}]",
            self.details, self.line, self.column
        )
    }
}

impl Error {
    pub(crate) fn new(src: &str, location: usize, details: Details) -> Self {
        let (first, _) = src.split_at(location);
        let start_of_line = first.rfind('\n').unwrap_or(0);

        Self {
            line: first.matches('\n').count() + 1,
            column: location - start_of_line + 1,
            details,
        }
    }
}

#[derive(Debug)]
pub enum Details {
    UnexpectedChar(char),
    NoClosingQuoteForString,
}

impl Display for Details {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoClosingQuoteForString => "no closing quote for string".to_string(),
                Self::UnexpectedChar(c) => format!("unexpected char: {}", c),
            }
        )
    }
}
