use crate::lexer::Lexer;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}
