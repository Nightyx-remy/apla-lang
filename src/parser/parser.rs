use crate::{util::{file::SourceFile, position::{Positioned, Position}}, lexer::token::{Token, Keyword}, parser::{error::ParserError, node::{Node, ValueNode, Operator, FunctionCallParameter, VarType, FunctionDefinitionParameter}}};

pub struct Parser {
    src: SourceFile,
    tokens: Vec<Positioned<Token>>,
    index: usize
}

impl Parser {

    pub fn new(src: SourceFile, tokens: Vec<Positioned<Token>>) -> Self {
        Self {
            src,
            tokens,
            index: 0
        }
    }

    pub fn take(self) -> SourceFile {
        self.src
    }

    fn current(&self) -> Option<Positioned<Token>> {
        return self.tokens.get(self.index).cloned();
    }

    fn expect_current(&self, token: Option<Token>, should_be: Option<String>) -> Result<Positioned<Token>, ParserError> {
        if let Some(current) = self.current() {
            if let Some(token) = token {
                if current.data == token {
                    return Ok(current);
                } else {
                    return Err(ParserError::UnexpectedToken(current, should_be));
                }
            } else {
                return Ok(current);
            }
        } else {
            return Err(ParserError::UnexpectedEOF(should_be));
        }
    } 

    fn expect_end_of_statement(&self) -> Result<(), ParserError> {
        if let Some(current) = self.current() {
            if current.data == Token::NewLine {
                return Ok(());
            } else {
                return Err(ParserError::UnexpectedToken(current, Some("NewLine or EOF".to_string())));
            }
        } 
        Ok(())
    }

    fn expect_identifier(&self) -> Result<Positioned<String>, ParserError> {
        if let Some(current) = self.current() {
            match &current.data {
                Token::Identifier(id) => return Ok(current.convert(id.clone())),
                _ => return Err(ParserError::UnexpectedToken(current, Some("Identifier".to_string())))
            }
        } else {
            return Err(ParserError::UnexpectedEOF(Some("Identifier".to_string())));
        }
    }

    fn expect_string(&self) -> Result<Positioned<String>, ParserError> {
        if let Some(current) = self.current() {
            match &current.data {
                Token::String(id) => return Ok(current.convert(id.clone())),
                _ => return Err(ParserError::UnexpectedToken(current, Some("String".to_string())))
            }
        } else {
            return Err(ParserError::UnexpectedEOF(Some("Identifier".to_string())));
        }
    }

    fn peek(&self, x: usize) -> Option<Positioned<Token>> {
        return self.tokens.get(self.index + x).cloned();
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn advance_x(&mut self, x: usize) {
        self.index += x;
    }

    fn parse_expr0(&mut self) -> Result<Positioned<Node>, ParserError> {
        let current = self.current();
        if let Some(current) = &current {
            match &current.data {
                Token::Decimal(value) => Ok(current.convert(Node::Value(ValueNode::Decimal(value.clone())))),
                Token::String(value) => Ok(current.convert(Node::Value(ValueNode::String(value.clone())))),
                Token::Identifier(id) => self.handle_identifier(current.convert(id.clone())),
                Token::Keyword(Keyword::This) => Ok(current.convert(Node::Value(ValueNode::This))),
                _ => Err(ParserError::UnexpectedToken(current.clone(), Some("expr0".to_string())))
            }
        } else {
            Err(ParserError::UnexpectedEOF(Some("expr0".to_string())))
        }
    }

    fn handle_identifier(&mut self, identifier: Positioned<String>) -> Result<Positioned<Node>, ParserError> {
        let next = self.peek(1);
        if next.is_none() || next.unwrap().data != Token::LeftParenthesis {
            // Variable Call
            let name = identifier.data.clone();
            return Ok(identifier.convert(Node::Value(ValueNode::VariableCall(name))));
        }

        self.advance_x(2);
        let mut current = self.expect_current(None, Some(")".to_string()))?;

        let mut params = Vec::new();
        while current.data != Token::RightParenthesis {
            if params.len() != 0 {
                self.expect_current(Some(Token::Comma), Some(",".to_string()))?;
                self.advance();
            }
            let value = self.parse_expr()?;
            params.push(FunctionCallParameter { value });
            current = self.expect_current(None, Some(")".to_string()))?;
        }
        let end = current.end;
        let start = identifier.start.clone();

        return Ok(Positioned::new(Node::FunctionCall { 
            name: identifier.clone(), 
            params 
        }, start, end));
    }

    fn parse_expr1(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr0()?;

        loop {
            self.advance();
            let Some(current) = self.current() else {
                break;
            };

            let op = match current.data {
                Token::Dot => current.convert(Operator::MemberAccess),
                _ => break
            };
            self.advance();

            let right = self.parse_expr0()?;
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { lhs: Box::new(left), op, rhs: Box::new(right) }, start, end);
        }

        Ok(left)
    }

    fn parse_expr2(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr1()?;

        loop {
            let Some(current) = self.current() else {
                break;
            };

            let op = match current.data {
                Token::Star => current.convert(Operator::Multiply),
                Token::Slash => current.convert(Operator::Divide),
                _ => break
            };
            self.advance();

            let right = self.parse_expr1()?;
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { lhs: Box::new(left), op, rhs: Box::new(right) }, start, end);
        }

        Ok(left)
    }

    fn parse_expr3(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr2()?;

        loop {
            let Some(current) = self.current() else {
                break;
            };

            let op = match current.data {
                Token::Plus => current.convert(Operator::Plus),
                Token::Dash => current.convert(Operator::Minus),
                _ => break
            };
            self.advance();

            let right = self.parse_expr2()?;
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { lhs: Box::new(left), op, rhs: Box::new(right) }, start, end);
        }

        Ok(left)
    }

    fn parse_expr4(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr3()?;

        loop {
            let Some(current) = self.current() else {
                break;
            };

            let op = match current.data {
                Token::Equal => current.convert(Operator::Assignment),
                _ => break
            };
            self.advance();

            let right = self.parse_expr3()?;
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { lhs: Box::new(left), op, rhs: Box::new(right) }, start, end);
        }

        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Positioned<Node>, ParserError> {
        self.parse_expr4()
    }

    fn parse_variable_definition(&mut self, var_type: Positioned<VarType>) -> Result<Positioned<Node>, ParserError> {
        self.advance();

        // Get Identifier
        let name = self.expect_identifier()?;
        self.advance();
        let mut end = name.end.clone();

        // Get Type
        let mut current = self.current();
        let mut data_type = None;
        if let Some(current) = current {
            if current.data == Token::Colon {
                self.advance();
                data_type = Some(self.expect_identifier()?);
                self.advance();
                end = data_type.as_ref().unwrap().end.clone();
            }
        }

        // Get Value
        current = self.current();
        let mut value = None;
        if let Some(current) = current {
            if current.data == Token::Equal {
                self.advance();
                value = Some(Box::new(self.parse_expr()?));
                end = value.as_ref().unwrap().end.clone();
            } 
        }
        
        let start = var_type.start.clone();

        Ok(Positioned::new(Node::VariableDefinition { 
            var_type, 
            name, 
            data_type, 
            value 
        }, start, end))        
    }

    fn parse_function_definition(&mut self, start: Position, external: bool, constructor: bool) -> Result<Positioned<Node>, ParserError> {
        self.advance();

        // Get name
        let name = self.expect_identifier()?;
        self.advance();

        // Get parameters
        self.expect_current(Some(Token::LeftParenthesis), Some("(".to_string()))?;
        self.advance();
        let mut params = Vec::new();
        let mut current = self.expect_current(None, Some(")".to_string()))?;
        while current.data != Token::RightParenthesis {
            if params.len() != 0 {
                self.expect_current(Some(Token::Comma), Some(",".to_string()))?;
                self.advance();
            }

            let name = self.expect_identifier()?;
            self.advance();
            self.expect_current(Some(Token::Colon), Some(":".to_string()))?;
            self.advance();
            let data_type= self.expect_identifier()?;

            params.push(FunctionDefinitionParameter { name, data_type });

            self.advance();
            current = self.expect_current(None, Some(")".to_string()))?;
        }
        let mut end = current.end.clone();
        self.advance();

        // Get type
        let mut data_type = None;
        if !constructor {
            let current = self.current();
            if let Some(current) = current {
                if current.data == Token::Colon {
                    self.advance();
                    data_type = Some(self.expect_identifier()?);
                    self.advance();
                    end = data_type.as_ref().unwrap().end.clone();
                }
            } else {
                if !external {
                    return Err(ParserError::UnexpectedEOF(Some("=>".to_string()))); 
                }
            }
        }

        // Get body
        let body = if !external {
            let mut body = Vec::new();

            self.expect_current(Some(Token::RightDoubleArrow), Some("=>".to_string()))?;
            self.advance();
            let mut current_opt = self.current();
            
            while let Some(current) = &current_opt {
                if current.data == Token::NewLine {
                    self.advance();
                    current_opt = self.current();
                    continue;
                }

                if current.data != Token::Tab {
                    break;
                } 

                self.advance();
                
                let node = self.parse_current()?;
                end = node.end.clone();
                body.push(node);
                current_opt = self.current();
            }

            Some(body)
        } else {
            None
        };

        Ok(Positioned::new(Node::FunctionDefinition { 
            name, 
            return_type: data_type, 
            params, 
            body,
            constructor 
        }, start, end))
    }

    fn parse_return(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let expr = self.parse_expr()?;
        let end = expr.end.clone();
        Ok(Positioned::new(Node::Return(Box::new(expr)), start, end))
    }

    fn parse_include(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let path = self.expect_string()?;
        let end = path.end.clone();
        self.advance();
        Ok(Positioned::new(Node::Include(path), start, end))
    }

    fn parse_class_definition(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();

        // Body
        let name = self.expect_identifier()?;
        self.advance();

        // Body
        let mut body = Vec::new();
        let mut end = name.end.clone();

        let mut current_opt = self.current();
        while let Some(current) = &current_opt {
            if current.data == Token::NewLine {
                self.advance();
                current_opt = self.current();
                continue;
            }

            if current.data != Token::Tab {
                break;
            } 

            self.advance();
            
            let node = self.parse_current()?;
            end = node.end.clone();
            body.push(node);
            current_opt = self.current();
        }

        Ok(Positioned::new(Node::ClassDefinition { name, body }, start, end))
    }

    fn handle_keyword(&mut self, keyword: Positioned<Keyword>) -> Result<Positioned<Node>, ParserError> {
        match keyword.data {
            Keyword::Fn => self.parse_function_definition(keyword.start.clone(), false, false),
            Keyword::Const => {
                let res = self.parse_variable_definition(keyword.convert(VarType::Constant))?;
                self.expect_end_of_statement()?;
                self.advance();
                Ok(res)
            }
            Keyword::Var => {
                let res = self.parse_variable_definition(keyword.convert(VarType::Variable))?;
                self.expect_end_of_statement()?;
                self.advance();
                Ok(res)
            }
            Keyword::Return => {
                let res = self.parse_return(keyword.start.clone())?;
                self.expect_end_of_statement()?;
                self.advance();
                Ok(res)
            }
            Keyword::Extern => {
                let start = keyword.start.clone();
                self.advance();
                self.expect_current(Some(Token::Keyword(Keyword::Fn)), Some("fn".to_string()))?;
                self.parse_function_definition(start, true, false)
            },
            Keyword::Include => {
                let res = self.parse_include(keyword.start.clone())?;
                self.expect_end_of_statement()?;
                self.advance();
                Ok(res)
            },
            Keyword::Class => self.parse_class_definition(keyword.start.clone()),
            Keyword::This => self.parse_expr(),
            Keyword::New => self.parse_function_definition(keyword.start.clone(), false, true),
        }
    } 

    fn parse_current(&mut self) -> Result<Positioned<Node>, ParserError> {
        let current = self.current();
        if let Some(current) = current {
            match &current.data {
                Token::Decimal(_) |
                Token::String(_) |
                Token::Identifier(_) => {
                    let res = self.parse_expr()?;
                    self.expect_end_of_statement()?;
                    self.advance();
                    Ok(res)
                }
                Token::Keyword(keyword) => self.handle_keyword(current.convert(keyword.clone())),
                Token::Plus |
                Token::Dash => todo!("Unary"),
                Token::LeftParenthesis => todo!("expr - parenthesis"),
                Token::NewLine | Token::Tab => {
                    self.advance();
                    self.parse_current()
                }
                _ => Err(ParserError::UnexpectedToken(current, None))
            }
        } else {
            Err(ParserError::UnexpectedEOF(None))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Positioned<Node>>, ParserError> {
        let mut ast = Vec::new();

        let mut current = self.current();
        while current.is_some() {
            while let Some(current_tok) = &current {
                match current_tok.data {
                    Token::NewLine |
                    Token::Tab => self.advance(),
                    _ => break
                }
                current = self.current();
            }
            if current.is_none() { break; }
            ast.push(self.parse_current()?);
            current = self.current();
        } 

        Ok(ast)
    }

}