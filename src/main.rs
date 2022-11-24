use std::collections::LinkedList;

use crate::{util::file::SourceFile, lexer::lexer::Lexer, parser::parser::Parser};

pub mod util;
pub mod lexer;
pub mod parser;
pub mod translator;

fn main() {
    let src = match std::fs::read_to_string("res/main.apla") {
        Ok(src) => src,
        Err(err) => {
            println!("Failed to read file '{}'\n{}", "res/main.apla", err);
            return;
        }
    };

    let mut src = SourceFile::new("main".to_string(), src);

    let mut lexer = Lexer::new(src);

    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            src = lexer.take();
            err.print_error(src);
            return;
        },
    };

    src = lexer.take();
    for token in tokens.iter() {
        println!("{}\n", token.data);
    }

    let mut parser = Parser::new(src, tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            src = parser.take();
            err.print_error(src);
            return;
        },
    };

    src = parser.take();
    for node in ast.iter() {
        println!("{}\n{}\n", node.data, node.arrow_message(&src.src));
    }
}
