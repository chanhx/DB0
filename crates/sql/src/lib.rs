pub mod catalog;
mod common;
mod error;
mod lexer;
pub mod logical_plan;
pub mod parser;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
