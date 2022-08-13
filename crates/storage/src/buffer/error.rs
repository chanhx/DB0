pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Internal {
        details: String,
        source: Option<Box<dyn std::error::Error>>,
    },
    BufferPoolIsFull,
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
                Self::BufferPoolIsFull => format!("buffer pool is full"),
            }
        )
    }
}
