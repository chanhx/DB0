#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Keyword(Keyword),

    Identifier,

    Number { is_float: bool },
    String,

    Comma,
    Period,
    Semicolon,
    LeftParen,
    RightParen,
    Question,

    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    LessOrGreaterThan,

    Caret,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
}

macro_rules! keyword {
    ( $( $var:ident, )* ) => {
        #[derive(Debug, PartialEq)]
        #[allow(non_camel_case_types)]
        pub(crate) enum Keyword {
            $($var,)*
        }

        #[derive(Debug)]
        pub(crate) struct NotKeywordError {}

        impl std::fmt::Display for NotKeywordError {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "not a keyword")
            }
        }
        impl std::error::Error for NotKeywordError {}

        impl std::str::FromStr for Keyword {
            type Err = NotKeywordError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_ascii_uppercase().as_str() {
                    $(stringify!($var) => Ok(Self::$var),)*
                    _=> Err(NotKeywordError{}),
                }
            }
        }
    };
}

keyword! {
    AND,
    AS,
    ASC,
    BEGIN,
    BIGINT,
    BOOLEAN,
    BY,
    CHAR,
    COMMIT,
    CREATE,
    CROSS,
    DATABASE,
    DECIMAL,
    DESC,
    DISTINCT,
    DROP,
    EXISTS,
    FALSE,
    FLOAT,
    FROM,
    IF,
    IN,
    INDEX,
    INNER,
    INSERT,
    INT,
    INTEGER,
    INTO,
    IS,
    JOIN,
    KEY,
    LEFT,
    LIKE,
    LIMIT,
    NOT,
    NUMERIC,
    NULL,
    ON,
    OR,
    ORDER,
    PRIMARY,
    REPLACE,
    RIGHT,
    ROLLBACK,
    SELECT,
    SMALLINT,
    TABLE,
    TEMP,
    TEMPORARY,
    TRUE,
    UNIQUE,
    UPDATE,
    VALUES,
    VARCHAR,
    WHERE,
}
