use {crate::common::Span, std::fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    DuplicateColumn { span: Span, details: String },
    Internal(String),
    MultiplePrimaryKey { span: Span, details: String },
    NoClosingQuoteForString(Span),
    UndefinedColumn { span: Span, details: String },
    UnexpectedChar { c: char, location: usize },
    UnexpectedEnd,
    RelationNotExist { name: String },
    SyntaxError(Span),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DuplicateColumn { details, .. } => details.clone(),
                Self::Internal(details) => details.clone(),
                Self::MultiplePrimaryKey { details, .. } => details.clone(),
                Self::NoClosingQuoteForString(_) => "no closing quote for string".to_string(),
                Self::UndefinedColumn { details, .. } => details.clone(),
                Self::UnexpectedChar { c, .. } => format!("unexpected char: {}", c),
                Self::UnexpectedEnd => "unexpected end of input".to_string(),
                Self::RelationNotExist { name } => format!("relation {} does not exists", name),
                Self::SyntaxError(_) => "syntax error".to_string(),
            }
        )
    }
}
