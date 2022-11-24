use std::fmt::{Display, Debug};

#[derive(Clone)]
pub struct Position {
    pub index: usize,
    pub column: usize,
    pub column_index: usize,
    pub line: usize
}

impl Default for Position {
    
    fn default() -> Self {
        Self { 
            index: 0, 
            column: 0, 
            column_index: 0, 
            line: 1
        }
    }

}

impl Position {

    pub fn advance(&mut self, chr: char) {
        if chr == '\n' {
            self.line += 1;
            self.column = 0;
            self.column_index = 0;
        } else if chr == '\t' {
            self.column += 4;
            self.column_index += 1;
        } else {
            self.column += 1;
            self.column_index += 1;
        }
        self.index += 1;
    } 

}

impl Display for Position {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }

}

pub struct Positioned<T> {
    pub start: Position,
    pub end: Position,
    pub data: T
}

impl<T> Positioned<T> {

    pub fn new(data: T, start: Position, end: Position) -> Positioned<T> {
        Self {
            start,
            end,
            data
        }
    }

    pub fn convert<U>(&self, data: U) -> Positioned<U> {
        return Positioned {
            start: self.start.clone(),
            end: self.end.clone(),
            data
        };
    }

    pub fn arrow_message(&self, src: &str) -> String {
        let mut buf: String = String::new();

        let mut lines = src.lines();
        let mut line = lines.nth(self.start.line - 1).unwrap();
        let mut index = self.start.line;
        while index <= self.end.line {
            let start = if index == self.start.line {
                self.start.column
            } else {
                0
            };
            let end = if index == self.end.line {
                if self.end.column < start { break; }
                self.end.column - start
            } else {
                if line.len() < start { break; }
                line.len() - start
            };

            if index != self.start.line {
                buf.push('\n');
            }

            buf.push_str(line.replace("\t", "    ").as_str());
            buf.push('\n');

            buf.push_str(" ".repeat(start).as_str());
            buf.push_str("^".repeat(end).as_str());

            if let Some(l) = lines.next() {
                line = l;
            } else {
                break;
            }
            index += 1;
        }

        return buf;
    }

}

impl<T: Display> Debug for Positioned<T> {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }

}


impl <T: Clone> Clone for Positioned<T> {
    fn clone(&self) -> Self {
        Self { start: self.start.clone(), end: self.end.clone(), data: self.data.clone() }
    }
}