use crate::common::Span;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    NoClosingQuoteForString(Span),
    UnexpectedChar { c: char, location: usize },
    UnexpectedEnd,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoClosingQuoteForString(_) => "no closing quote for string".to_string(),
                Self::UnexpectedChar { c, .. } => format!("unexpected char: {}", c),
                Self::UnexpectedEnd => "unexpected end of input".to_string(),
            }
        )
    }
}
