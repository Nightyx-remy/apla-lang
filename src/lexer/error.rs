use crate::util::{position::Positioned, file::SourceFile};

pub enum LexerError {
    UnexpectedEOF,
    UnexpectedChar(Positioned<char>)
}

impl LexerError {

    pub fn print_error(&self, src: SourceFile) {
        match self {
            LexerError::UnexpectedEOF => println!("[Lexer]: Unexpected EOF."),
            LexerError::UnexpectedChar(chr) => println!("[Lexer]: Unexpected char {:?} at {} in {}.apla\n{}", chr.data, chr.start, src.name, chr.arrow_message(&src.src)),
        }
    }

}