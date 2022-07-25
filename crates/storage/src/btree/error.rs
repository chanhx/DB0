pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Internal {
        details: String,
        source: Option<Box<dyn std::error::Error>>,
    },
    KeyAlreadyExists(String),
    KeyNotFound(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
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
                Self::KeyAlreadyExists(key) => format!("{} already exists", key),
                Self::KeyNotFound(key) => format!("{} not found", key),
            }
        )
    }
}
