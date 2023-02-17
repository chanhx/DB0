use ast::Span;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    DuplicateColumn { span: Span, details: String },
    MultiplePrimaryKey { span: Span, details: String },
    UndefinedColumn { span: Span, details: String },
    Unimplemented,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DuplicateColumn { details, .. } => details.clone(),
                Self::MultiplePrimaryKey { details, .. } => details.clone(),
                Self::UndefinedColumn { details, .. } => details.clone(),
                Self::Unimplemented => "statement is not supported yet".to_string(),
            }
        )
    }
}
