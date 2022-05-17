use std::ops::RangeInclusive;

#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    Keyword(Keyword),

    Identifier,

    Number,
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

    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
}

pub(crate) type Span = RangeInclusive<usize>;

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
    ASC,
    BEGIN,
    BY,
    COMMIT,
    CREATE,
    DATABASE,
    DESC,
    EXISTS,
    FROM,
    IF,
    IN,
    INDEX,
    INNER,
    INSERT,
    INTO,
    IS,
    JOIN,
    KEY,
    LEFT,
    LIMIT,
    ON,
    OR,
    ORDER,
    NOT,
    RIGHT,
    ROLLBACK,
    SELECT,
    TABLE,
    UPDATE,
    VALUES,
    WHERE,
}
