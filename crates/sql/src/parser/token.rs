#[derive(Debug, PartialEq)]
pub(super) enum Token<'a> {
    Keyword(Keyword),

    Identifier(&'a str),

    Number(&'a str),
    String(&'a str),

    Comma,
    Period,
    Semicolon,
    LeftParen,
    RightParen,

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

#[derive(Debug, PartialEq)]
pub(super) enum Keyword {
    And,
    Asc,
    Begin,
    By,
    Commit,
    Create,
    Database,
    Desc,
    From,
    In,
    Index,
    Inner,
    Insert,
    Into,
    Is,
    Join,
    Key,
    Left,
    Limit,
    On,
    Or,
    Order,
    Right,
    Rollback,
    Select,
    Table,
    Update,
    Values,
    Where,
}

impl Keyword {
    pub(super) fn from_str(s: &str) -> Option<Self> {
        Some(match s.to_ascii_uppercase().as_str() {
            "AND" => Self::And,
            "ASC" => Self::Asc,
            "BEGIN" => Self::Begin,
            "BY" => Self::By,
            "COMMIT" => Self::Commit,
            "CREATE" => Self::Create,
            "DATABASE" => Self::Database,
            "DESC" => Self::Desc,
            "FROM" => Self::From,
            "IN" => Self::In,
            "INDEX" => Self::Index,
            "INNER" => Self::Inner,
            "INSERT" => Self::Insert,
            "INTO" => Self::Into,
            "IS" => Self::Is,
            "JOIN" => Self::Join,
            "KEY" => Self::Key,
            "LEFT" => Self::Left,
            "LIMIT" => Self::Limit,
            "ON" => Self::On,
            "OR" => Self::Or,
            "ORDER" => Self::Order,
            "RIGHT" => Self::Right,
            "ROLLBACK" => Self::Rollback,
            "SELECT" => Self::Select,
            "TABLE" => Self::Table,
            "UPDATE" => Self::Update,
            "VALUES" => Self::Values,
            "WHERE" => Self::Where,
            _ => return None,
        })
    }
}
