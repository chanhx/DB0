pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO {
        source: std::io::Error,
    },
    Internal {
        details: String,
        source: Option<Box<dyn std::error::Error>>,
    },
    InvalidPageType(u8),
    KeyAlreadyExists,
    KeyNotFound(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IO { source } => format!("IO error: {}", source),
                Self::Internal { details, source } => {
                    format!(
                        "internal error: {}{}",
                        details,
                        match source {
                            Some(err) => format!(", source: {}", err),
                            None => "".to_string(),
                        }
                    )
                }
                Self::InvalidPageType(ty) => format!("invalid page type {}", ty),
                Self::KeyAlreadyExists => "key already exists".to_string(),
                Self::KeyNotFound(key) => format!("{} not found", key),
            }
        )
    }
}
