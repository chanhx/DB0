use {crate::lexer, ast::Span};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedEnd,
    SyntaxError(Span),
    LexingError(lexer::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::UnexpectedEnd => "unexpected end of input".to_string(),
                Self::SyntaxError(_) => "syntax error".to_string(),
                Self::LexingError(e) => e.to_string(),
            }
        )
    }
}
