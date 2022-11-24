use std::default;

use crate::{util::{file::SourceFile, position::{Position, Positioned}}, lexer::{token::{Token, Keyword}, error::LexerError}};

pub struct Lexer {
    src: SourceFile,
    chars: Vec<char>,
    pos: Position
}

impl Lexer {

    pub fn new(src: SourceFile) -> Lexer {
        let chars = src.src.chars().collect();
        return Self {
            src,
            chars,
            pos: Position::default()
        }
    }

    fn current(&self) -> char {
        return self.chars.get(self.pos.index).map(|chr| *chr).unwrap_or('\0');
    } 

    fn peek(&self, x: usize) -> char {
        return self.chars.get(self.pos.index + x).map(|chr| *chr).unwrap_or('\0');
    }

    fn advance(&mut self) {
        self.pos.advance(self.current());
    }

    fn make_single<T>(&self, data: T) -> Positioned<T> {
        let start = self.pos.clone();
        let mut end = self.pos.clone();
        end.advance(self.current());
        Positioned::new(data, start, end)
    }

    fn make_number(&mut self) -> Positioned<Token> {
        let mut buf = String::new();
        let start = self.pos.clone();

        let mut current = self.current();
        while current.is_digit(10) {
            buf.push(current);
            self.advance();
            current = self.current();
        }

        let end = self.pos.clone();

        return Positioned::new(Token::Decimal(buf), start, end);
    }

    fn make_identifier(&mut self) -> Positioned<Token> {
        let mut buf = String ::new();
        let start = self.pos.clone();

        let mut current = self.current();
        while current.is_alphabetic() {
            buf.push(current);
            self.advance();
            current = self.current();
        }

        let end = self.pos.clone();

        Positioned::new(if let Some(keyword) = Keyword::from_string(buf.clone()) {
            Token::Keyword(keyword)
        } else {
            Token::Identifier(buf)
        }, start, end)
    }

    fn make_string(&mut self) -> Result<Positioned<Token>, LexerError> {
        let mut buf = String::new();
        let start = self.pos.clone();
        self.advance();

        let mut current = self.current();
        while current != '"' {
            if current == '\0' {
                return Err(LexerError::UnexpectedEOF);
            }
            buf.push(current);
            self.advance();
            current = self.current();
        }
        self.advance();
        let end = self.pos.clone();

        return Ok(Positioned::new(Token::String(buf), start, end));
    }

    pub fn tokenize(&mut self) -> Result<Vec<Positioned<Token>>, LexerError> {
        let mut tokens = Vec::new();
        let mut space_count = 0;
        let mut space_start = self.pos.clone();

        loop {
            let mut current = self.current();
            
            space_count = 0;
            while current == ' ' {
                space_count += 1;
                if space_count == 1 {
                    space_start = self.pos.clone();
                } else if space_count == 4 {
                    space_count = 0;
                    let mut end = self.pos.clone();
                    end.advance(' ');
                    tokens.push(Positioned::new(Token::Tab, space_start, end));
                    space_start = self.pos.clone();
                }
                self.advance();
                current = self.current();
            }

            match current {
                '0'..='9'=> {
                    tokens.push(self.make_number());
                    continue;
                }
                'a'..='z' | 'A'..='Z' => {
                    tokens.push(self.make_identifier());
                    continue;
                }
                '"' => {
                    tokens.push(self.make_string()?);
                    continue;
                }
                '+' => tokens.push(self.make_single(Token::Plus)),
                '-' => tokens.push(self.make_single(Token::Dash)),
                '*' => tokens.push(self.make_single(Token::Star)),
                '/' => tokens.push(self.make_single(Token::Slash)),
                '(' => tokens.push(self.make_single(Token::LeftParenthesis)),
                ')' => tokens.push(self.make_single(Token::RightParenthesis)),
                ':' => tokens.push(self.make_single(Token::Colon)),
                ',' => tokens.push(self.make_single(Token::Comma)),
                '\n' => {
                    let start = self.pos.clone();
                    let mut end = self.pos.clone();
                    end.advance(' ');
                    tokens.push(Positioned::new(Token::NewLine, start, end));
                }
                '\t' => tokens.push(self.make_single(Token::Tab)),
                '=' => {
                    if self.peek(1) == '>' {
                        let start = self.pos.clone();
                        self.advance();
                        let mut end = self.pos.clone();
                        end.advance(self.current());
                        tokens.push(Positioned::new(Token::RightDoubleArrow, start, end));
                    } else {
                        tokens.push(self.make_single(Token::Equal));
                    }
                }
                '#' => {
                    while current != '\n' && current != '\0' {
                        self.advance();
                        current = self.current();
                    }
                }
                '\0' => break,
                _ => return Err(LexerError::UnexpectedChar(self.make_single(current)))
            }
            self.advance();
        }      

        Ok(tokens)
    }

    pub fn take(self) -> SourceFile {
        return self.src;
    }

}