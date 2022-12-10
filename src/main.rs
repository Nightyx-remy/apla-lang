use crate::{util::file::SourceFile, lexer::lexer::Lexer, parser::parser::Parser, translator::translator::Translator, checker::checker::Checker};

pub mod util;
pub mod lexer;
pub mod parser;
pub mod translator;
pub mod checker;

// TODO: Create full project with file structure and CMakeList.txt
// FIXME: Fix issue with TAB after new line in function body
// FIXME: Add TAB count for function
// FIXME: Consider issue with includes in header file (because of pushing at the beginning) [do the same thing as the checker (post processing)]
// TODO: Add errors to transpile_project()
// TODO: Checker errors
// TODO: Add Destructor
// TODO: rename functions and fields depending on the class
// TODO: transform non-c-type to pointers [checker]
// FIXME: Change c_byte to c_char

fn transpile_project(folder: &str) {    
    std::fs::remove_dir_all("./out").unwrap();
    std::fs::create_dir("./out").unwrap();

    for file in std::fs::read_dir(folder).unwrap() {
        let file = file.unwrap();

        if file.file_type().unwrap().is_file() {
            let mut name = file.file_name().to_str().unwrap().to_string();
            if !name.ends_with(".apla") {
                continue;
            }
            name = (&name[0..(name.len() - 5)]).to_string();
            
            let src = std::fs::read_to_string(file.path()).unwrap();

            let mut src = SourceFile::new(name, src);

            println!("\n\n--- Lexer ---");
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
        
            println!("\n\n--- Parser ---");
            let mut parser = Parser::new(src, tokens);
            let mut ast = match parser.parse() {
                Ok(ast) => ast,
                Err(err) => {
                    src = parser.take();
                    err.print_error(src);
                    return;
                },
            };
        
            src = parser.take();
            for node in ast.iter() {
                println!("{}\n", node.data);
                // println!("{}\n", node.arrow_message(&src.src));
            }

            println!("\n\n--- Checker ---");
            let mut checker = Checker::new(src, ast);
            ast = checker.check();
            src = checker.take();

            
            for node in ast.iter() {
                println!("{}\n", node.data);
                // println!("{}\n", node.arrow_message(&src.src));
            }
        
            println!("\n\n--- Translator ---");
            let mut translator = Translator::new(src, ast);
            let project = translator.translate();
            src = translator.take();
        
            println!("\n\n");
            for file in project.files.iter() {
                if !file.header.is_empty() {
                    std::fs::write(format!("./out/{}.h", file.name), &file.header).unwrap();
                    // println!("{}.h", file.name);
                    // println!("{}\n", file.header);
                }
                if !file.src.is_empty() {
                    std::fs::write(format!("./out/{}.c", file.name), &file.src).unwrap();
                    // println!("{}.c", file.name);
                    // println!("{}\n", file.src);
                }
            }
        }
    }
} 

fn main() {
    transpile_project("./res");
}
