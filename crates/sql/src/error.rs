use {crate::Span, std::fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Error {
    span: Span,
    details: Details,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error {
    pub(crate) fn new(span: Span, details: Details) -> Self {
        Self { span, details }
    }
}

#[derive(Debug, PartialEq)]
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
