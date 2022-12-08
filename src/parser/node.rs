use std::fmt::Display;

use crate::util::position::Positioned;

#[derive(Clone)]
pub enum Node {
    Value(ValueNode),
    BinaryOperation {
        lhs: Box<Positioned<Node>>,
        op: Positioned<Operator>,
        rhs: Box<Positioned<Node>>
    },
    VariableDefinition {
        var_type: Positioned<VarType>,
        name: Positioned<String>,
        data_type: Option<Positioned<String>>,
        value: Option<Box<Positioned<Node>>>
    },
    FunctionDefinition {
        name: Positioned<String>,
        return_type: Option<Positioned<String>>,
        params: Vec<FunctionDefinitionParameter>,
        body: Option<Vec<Positioned<Node>>>,
        constructor: bool,
    }, 
    Return(Box<Positioned<Node>>),
    FunctionCall {
        name: Positioned<String>,
        params: Vec<FunctionCallParameter>,
    },
    Include (Positioned<String>),
    ClassDefinition {
        name: Positioned<String>,
        body: Vec<Positioned<Node>>
    }
}

impl Display for Node {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Value(value) => write!(f, "{}", value)?,
            Node::BinaryOperation { lhs, op, rhs } => {
                write!(f, "({}", lhs.data)?;
                match op.data {
                    Operator::Plus => write!(f, " + ")?,
                    Operator::Minus => write!(f, " - ")?,
                    Operator::Multiply => write!(f, " * ")?,
                    Operator::Divide => write!(f, " / ")?,
                    Operator::MemberAccess => write!(f, ".")?,
                    Operator::Assignment => write!(f, " = ")?,
                }
                write!(f, "{})", rhs.data)?;
            },
            Node::VariableDefinition { var_type, name, data_type, value } => {
                match var_type.data {
                    VarType::Constant => write!(f, "const ")?,
                    VarType::Variable => write!(f, "var ")?,
                }

                write!(f, "{}", name.data)?;

                if let Some(data_type) = data_type {
                    write!(f, ": {}", data_type.data)?;
                }

                if let Some(value) = value {
                    write!(f, " = {}", value.data)?;
                }
            },
            Node::FunctionDefinition { name, return_type, params, body, constructor } => {
                if body.is_none() {
                    write!(f, "extern ")?;
                }

                if *constructor {
                    write!(f, "new ")?;
                } else {
                    write!(f, "fn ")?;
                }
                write!(f, "{}(", name.data)?;

                // Parameters
                let mut i = 0;
                for param in params.iter() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}: {}", param.name.data, param.data_type.data)?;

                    i += 1;
                }
                write!(f, ")")?;

                // Return type
                if let Some(return_type) = return_type {
                    write!(f, ": {}", return_type.data)?;
                }

                if let Some(body) = body {
                    write!(f, " =>")?;
                    for node in body.iter() {
                        let str = node.data.to_string();
                        for line in str.lines() {
                            write!(f, "\n\t{}", line)?;
                        }
                    }
                }
            },
            Node::Return(node) => write!(f, "return {}", node.data)?,
            Node::FunctionCall { name, params } => {
                write!(f, "{}(", name.data)?;
                let mut i = 0;
                for param in params.iter() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param.value.data)?;
                    i += 1;
                }
                write!(f, ")")?;
            },
            Node::Include(path) => write!(f, "include \"{}\"", path.data)?,
            Node::ClassDefinition { name, body } => {
                write!(f, "class {}", name.data)?;

                for node in body.iter() {
                    let str = node.data.to_string();
                    for line in str.lines() {
                        write!(f, "\n\t{}", line)?;
                    }
                }
            },
        }
        Ok(())
    }

}

#[derive(Clone)]
pub enum ValueNode {
    Decimal(String),
    String(String),
    VariableCall(String),
    This,
}

impl Display for ValueNode {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueNode::Decimal(val) => write!(f, "{}", val),
            ValueNode::String(val) => write!(f, "\"{}\"", val),
            ValueNode::VariableCall(name) => write!(f, "{}", name),
            ValueNode::This => write!(f, "self"),
        }
    }

}

#[derive(Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    MemberAccess,
    Assignment
}

#[derive(Clone)]
pub enum VarType {
    Constant,
    Variable
}

#[derive(Clone)]
pub struct FunctionDefinitionParameter {
    pub name: Positioned<String>,
    pub data_type: Positioned<String>
}

#[derive(Clone)]
pub struct FunctionCallParameter {
    pub value: Positioned<Node>,
}