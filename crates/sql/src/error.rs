use {crate::common::Span, std::fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedChar { c: char, location: usize },
    NoClosingQuoteForString(Span),
    UnexpectedEnd,
    SyntaxError(Span),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoClosingQuoteForString(_) => "no closing quote for string".to_string(),
                Self::UnexpectedChar { c, .. } => format!("unexpected char: {}", c),
                Self::UnexpectedEnd => "unexpected end of input".to_string(),
                Self::SyntaxError(_) => "syntax error".to_string(),
            }
        )
    }
}
