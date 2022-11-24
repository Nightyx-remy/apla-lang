use std::fmt::Display;

#[derive(Clone, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    Const,
    Var,
    Return,
    Extern,
    Include
}

impl Keyword {

    pub fn from_string(str: String) -> Option<Keyword> {
        match str.as_str() {
            "fn" => Some(Keyword::Fn),
            "const" => Some(Keyword::Const),
            "var" => Some(Keyword::Var),
            "return" => Some(Keyword::Return),
            "extern" => Some(Keyword::Extern),
            "include" => Some(Keyword::Include),
            _ => None
        }
    }

}

impl Display for Keyword {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Fn => write!(f, "fn"),
            Keyword::Const => write!(f, "const"),
            Keyword::Var => write!(f, "var"),
            Keyword::Return => write!(f, "return"),
            Keyword::Extern => write!(f, "extern"),
            Keyword::Include => write!(f, "include"),
        }
    }

}

#[derive(Clone, PartialEq, Eq)]
pub enum Token {
    Decimal(String),
    String(String),
    Identifier(String),
    Keyword(Keyword),
    Plus,
    Dash,
    Star,
    Slash,
    Equal,
    Colon,
    Comma,
    LeftParenthesis,
    RightParenthesis,
    RightDoubleArrow,
    NewLine,
    Tab
}

impl Display for Token {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Decimal(val) => write!(f, "{}", val),
            Token::String(val) => write!(f, "\"{}\"", val),
            Token::Identifier(val) => write!(f, "{}", val),
            Token::Keyword(keyword) => write!(f, "Keyword({})", keyword),
            Token::Plus => write!(f, "+"),
            Token::Dash => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Equal => write!(f, "="),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::LeftParenthesis => write!(f, "("),
            Token::RightParenthesis => write!(f, ")"),
            Token::RightDoubleArrow => write!(f, "=>"),
            Token::NewLine => write!(f, "NewLine"),
            Token::Tab => write!(f, "Tab"),
        }
    }

}