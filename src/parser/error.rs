use crate::{util::{file::SourceFile, position::Positioned}, lexer::token::Token};

pub enum ParserError {
    UnexpectedEOF(Option<String>),
    UnexpectedToken(Positioned<Token>, Option<String>),
}

impl ParserError {

    pub fn print_error(&self, src: SourceFile) {
        match self {
            ParserError::UnexpectedEOF(should_be) => {
                println!("[Parser]: Unexpected EOF in '{}'", src.name);
                if let Some(should_be) = should_be {
                    println!(", should be '{}'", should_be);
                }
            },
            ParserError::UnexpectedToken(token, should_be) => {
                println!("[Parser]: Unexpected token '{}' at '{}' in '{}'", token.data, token.start, src.name);
                if let Some(should_be) = should_be {
                    println!(", should be '{}'", should_be);
                }
                println!("\n{}", token.arrow_message(&src.src));
            },
        }
    }

}