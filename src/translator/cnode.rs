use std::fmt::Display;

pub enum CNode {
    Value(CValueNode),
    BinaryOperation {
        lhs: Box<CNode>,
        op: COperator,
        rhs: Box<CNode>
    },
    VariableDefinition {
        data_type: String,
        name: String,
        value: Option<Box<CNode>>
    },
    FunctionDefinition {
        data_type: String,
        name: String,
        params: Vec<(String, String)>,
        body: Option<Vec<CNode>>
    },
    Return(Box<CNode>),
    FunctionCall {
        name: String,
        params: Vec<CNode>
    },
    Include(String)
}

impl Display for CNode {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CNode::Value(value) => write!(f, "{}", value)?,
            CNode::BinaryOperation { lhs, op, rhs } => {
                write!(f, "{}", lhs)?;

                match op {
                    COperator::Plus => write!(f, " + ")?,
                    COperator::Minus => write!(f, " - ")?,
                    COperator::Multiply => write!(f, " * ")?,
                    COperator::Divide => write!(f, " / ")?,
                }

                write!(f, "{}", rhs)?;
            },
            CNode::VariableDefinition { data_type, name, value } => {
                write!(f, "{} {}", data_type, name)?;
                if let Some(value) = value {
                    write!(f, " = {}", value)?;
                }
            },
            CNode::FunctionDefinition { data_type, name, params, body } => {
                write!(f, "{} {}(", data_type, name)?;
                let mut i = 0;
                for param in params {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", param.0, param.1)?;
                    i += 1;
                }
                if let Some(body) = body {
                    write!(f, ") {{ ")?;
                    for node in body {
                        let node_str = node.to_string();
                        i = 0;
                        for line in node_str.lines() {
                            if i == 0 {
                                write!(f, "\n")?;
                            }
                            write!(f, "\t{}\n", line)?;
                            i += 1;
                        }
                    }
                    write!(f, "}}")?;
                }
            },
            CNode::Return(expr) => write!(f, "return {}", expr)?,
            CNode::FunctionCall { name, params } => {
                write!(f, "{}(", name)?;
                let mut i = 0;
                for param in params {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                    i += 1;
                }
                write!(f, ")")?;
            },
            CNode::Include(path) => write!(f, "include \"{}\"", path)?,
        }
        
        Ok(())
    }

}

pub enum CValueNode {
    Decimal(String),
    String(String),
    VariableCall(String)
}

impl Display for CValueNode {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CValueNode::Decimal(val) => write!(f, "{}", val),
            CValueNode::String(val) => write!(f, "\"{}\"", val),
            CValueNode::VariableCall(name) => write!(f, "{}", name),
        }
    }

}

pub enum COperator {
    Plus,
    Minus,
    Multiply,
    Divide
}