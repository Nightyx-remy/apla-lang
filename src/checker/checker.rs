use core::panic;
use std::{fmt::Display, sync::{Arc, Mutex}};

use crate::{util::{file::SourceFile, position::Positioned}, parser::node::{Node, ValueNode, Operator, VarType, FunctionDefinitionParameter, FunctionCallParameter}};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DataType {
    Void,
    CDecimal,
    CString,
    Custom(String)
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Void => write!(f, "void")?,
            DataType::CDecimal => write!(f, "int")?,
            DataType::CString => todo!("C String"),
            DataType::Custom(custom) => write!(f, "{}", custom)?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct NodeInfo {
    data_type: Option<DataType>, // TODO: change to option and check every time to see if the datatype can be inferred
    symbol: Option<Symbol>
}

impl NodeInfo {

    pub fn new(data_type: Option<DataType>, symbol: Option<Symbol>) -> Self {
        Self {
            data_type,
            symbol
        }
    }

}

#[derive(Clone, Debug)]
pub struct VariableSymbol {
    var_type: VarType,
    name: String,
    data_type: Option<DataType>,
    initialized: bool,
}

impl VariableSymbol {

    pub fn new(var_type: VarType, name: String, data_type: Option<DataType>, initialized: bool) -> VariableSymbol {
        Self {
            var_type,
            name,
            data_type,
            initialized
        }
    }

}

#[derive(Clone, Debug)]
pub enum FunctionType {
    ExternalFunction,
    Constructor,
    Function
}

#[derive(Clone, Debug)]
pub struct FunctionSymbol {
    name: String,
    data_type: DataType,
    function_type: FunctionType,
    params: Vec<FunctionDefinitionParameter>
}

impl FunctionSymbol {

    pub fn new(name: String, data_type: DataType, function_type: FunctionType, params: Vec<FunctionDefinitionParameter>) -> FunctionSymbol {
        Self {
            name,
            data_type,
            function_type,
            params
        }
    }

}

#[derive(Clone, Debug)]
pub struct ClassSymbol {
    name: String,
    fields: Vec<Arc<Mutex<VariableSymbol>>>,
    functions: Vec<Arc<Mutex<FunctionSymbol>>>
}

impl ClassSymbol {

    pub fn new(name: String) -> ClassSymbol {
        Self {
            name,
            fields: Vec::new(),
            functions: Vec::new()
        }
    }

    pub fn get_field(&mut self, name: String) -> Option<Arc<Mutex<VariableSymbol>>> {
        for field in self.fields.iter() {
            if field.lock().unwrap().name == name {
                return Some(field.clone());
            }
        }

        None
    }

    pub fn get_function(&mut self, name: String) -> Option<Arc<Mutex<FunctionSymbol>>> {
        for function in self.functions.iter() {
            if function.lock().unwrap().name == name {
                return Some(function.clone());
            }
        }

        None
    }

}

#[derive(Clone, Debug)]
pub enum Symbol {
    Function(Arc<Mutex<FunctionSymbol>>),
    Variable(Arc<Mutex<VariableSymbol>>),
    Class(Arc<Mutex<ClassSymbol>>)
}

#[derive(Clone, Debug)]
pub enum ScopeType {
    Root,
    Function(String, DataType),
    Class(String)
}

#[derive(Clone, Debug)]
pub struct Scope {
    scope: ScopeType,
    parent: Option<Box<Scope>>,
    variables: Vec<Arc<Mutex<VariableSymbol>>>,
    functions: Vec<Arc<Mutex<FunctionSymbol>>>,
    classes: Vec<Arc<Mutex<ClassSymbol>>>,
    selected: Option<Box<Scope>>
}

impl Scope {

    pub fn new(scope: ScopeType, parent: Option<Box<Scope>>) -> Self {
        Self {
            scope,
            parent,
            variables: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            selected: None,
        }
    }

    pub fn symbol_exists(&mut self, name: String) -> bool {
        self.get_variable(name.clone()).is_some() || 
        self.get_function(name.clone()).is_some() || 
        self.get_class(name.clone()).is_some()
    }

    pub fn get_variable(&mut self, name: String) -> Option<Arc<Mutex<VariableSymbol>>> {
        if let Some(selected) = &mut self.selected {
            let var = selected.get_variable(name);
            self.selected = None;
            return var;
        } 

        for variable in self.variables.iter() {
            if variable.lock().unwrap().name == name {
                return Some(variable.clone());
            }
        }
        if let Some(parent) = &mut self.parent {
            parent.get_variable(name)
        } else {
            None
        }
    }

    pub fn get_function(&mut self, name: String) -> Option<Arc<Mutex<FunctionSymbol>>> {
        if let Some(selected) = &mut self.selected {
            let fun = selected.get_function(name);
            self.selected = None;
            return fun;
        } 

        for function in self.functions.iter() {
            if function.lock().unwrap().name == name {
                return Some(function.clone());
            }
        }
        if let Some(parent) = &mut self.parent {
            parent.get_function(name)
        } else {
            None
        }
    }

    pub fn get_class(&mut self, name: String) -> Option<Arc<Mutex<ClassSymbol>>> {
        if let Some(selected) = &mut self.selected {
            let class = selected.get_class(name);
            self.selected = None;
            return class;
        } 

        for class in self.classes.iter_mut() {
            if class.lock().unwrap().name == name {
                return Some(class.clone());
            }
        }
        if let Some(parent) = &mut self.parent {
            parent.get_class(name)
        } else {
            None
        }
    }

}

pub struct Checker {
    src: SourceFile,
    ast: Vec<Positioned<Node>>,
    index: usize,
    scope: Scope,
    includes: Vec<(Positioned<()>, Positioned<String>)>
}

impl Checker {

    pub fn new(src: SourceFile, ast: Vec<Positioned<Node>>) -> Self {
        Self {
            src,
            ast,
            index: 0,
            scope: Scope::new(ScopeType::Root, None),
            includes: Vec::new(),
        }
    }

    pub fn take(self) -> SourceFile {
        self.src
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current(&self) -> Option<Positioned<Node>> {
        self.ast.get(self.index).cloned()
    }

    fn check_data_type(&mut self, expected: DataType, found: DataType) -> bool {
        // TODO: Check if it can be explicitly casted (later)
        if expected == found {
            return true;
        }
        match expected {
            DataType::CDecimal => {
                if let DataType::Custom(inner) = &found {
                    match inner.as_str() {
                        "c_char" => return true,
                        "c_short" => return true,
                        "c_int" => return true,
                        "c_long" => return true,
                        "c_float" => return true,
                        "c_double" => return true,
                        _ => {}
                    }
                } 
            },
            DataType::CString => {
                if let DataType::Custom(inner) = &found {
                    if inner == "c_string" {
                        return true;
                    }
                }
            }
            _ => {}
        }
        match found {
            DataType::CDecimal => {
                if let DataType::Custom(inner) = expected {
                    match inner.as_str() {
                        "c_char" => true,
                        "c_short" => true,
                        "c_int" => true,
                        "c_long" => true,
                        "c_float" => true,
                        "c_double" => true,
                        _ => false
                    }
                } else {
                    false
                }
            },
            DataType::CString => {
                if let DataType::Custom(inner) = expected {
                    if inner == "c_string" {
                        return true;
                    }
                }
                false
            }
            _ => false
        }
    }

    pub fn infer_and_check(&mut self, node_info: NodeInfo, other: DataType) -> DataType {
        if let Some(data_type) = node_info.data_type.clone() {
            if self.check_data_type(other.clone(), data_type.clone()) {
                other
            } else {
                panic!("Unexpected type '{:?}', should be '{:?}'", data_type, other); 
            }
        } else if let Some(variable) = node_info.symbol {
            match variable {
                Symbol::Variable(variable) => {
                    if let Some(data_type) = variable.lock().unwrap().data_type.clone() {
                        if self.check_data_type(other.clone(), data_type) {
                            other
                        } else {
                            panic!("Unexpected type")
                        }
                    } else {
                        variable.lock().unwrap().data_type = Some(other.clone());
                        other
                    }
                },
                _ => panic!("Unexpected panic!")
            }
        } else {
            panic!("Unexpected panic!");
        }
    }

    pub fn infer_and_check2(&mut self, node_info: NodeInfo, other: NodeInfo) -> DataType {
        if let Some(data_type) = other.data_type {
            self.infer_and_check(node_info, data_type)
        } else if let Some(data_type) = node_info.data_type {
            self.infer_and_check(other, data_type)
        } else {
            panic!("Cannot infer type (constraints not supported)")
        }
    }

    fn check_value(&mut self, value_node: Positioned<ValueNode>) -> (NodeInfo, Vec<Positioned<Node>>) {
        match value_node.data.clone() {
            ValueNode::Decimal(_) => (
                NodeInfo::new(Some(DataType::CDecimal), None), 
                vec![value_node.convert(Node::Value(value_node.data.clone()))]
            ),
            ValueNode::String(_) => (
                NodeInfo::new(Some(DataType::CString), None), 
                vec![value_node.convert(Node::Value(value_node.data.clone()))]
            ),
            ValueNode::VariableCall(value) => {
                if let Some(variable) = self.scope.get_variable(value.clone()) {
                    (NodeInfo::new(variable.lock().unwrap().data_type.clone(), Some(Symbol::Variable(variable.clone()))), vec![
                        value_node.convert(Node::Value(ValueNode::VariableCall(value.clone())))
                    ])
                } else if let Some(class) = self.scope.get_class(value.clone()) {
                    (NodeInfo::new(Some(DataType::Custom(value.clone())), Some(Symbol::Class(class))), vec![
                        value_node.convert(Node::Value(ValueNode::VariableCall(value.clone())))
                    ])
                } else {
                    panic!("Variable / Class '{}' not found", value)
                }
            }
            ValueNode::This => {
                if let Some(this) = self.scope.get_variable("self".to_string()) {
                    let data_type = this.lock().unwrap().data_type.clone();
                    (NodeInfo::new(data_type, Some(Symbol::Variable(this))), vec![
                        value_node.convert(Node::Value(ValueNode::This))
                    ])
                } else {
                    panic!("Cannot use this outside of a class!");
                }
            },
        }
    }

    fn check_assignment(&mut self, position: Positioned<()>, lhs: Positioned<Node>, op: Positioned<Operator>, rhs: Positioned<Node>) -> (NodeInfo, Vec<Positioned<Node>>) {
        let (lhs_info, lhs_ast) = self.check_node(lhs);
        let (rhs_info, rhs_ast) = self.check_node(rhs);
        
        // Check var type
        if let Some(symbol) = &lhs_info.symbol {
            match symbol {
                Symbol::Function(_) => panic!("Cannot assign to functions!"),
                Symbol::Variable(variable) => {
                    let mut variable = variable.lock().unwrap();
                    if variable.var_type == VarType::Constant && variable.initialized {
                        panic!("Cannot assign to constant '{:?}'!", variable);
                    }
                    variable.initialized = true;
                },
                Symbol::Class(_) => panic!("Cannot assign to classes"),
            }
        }

        // Check data_type
        let data_type = self.infer_and_check2(lhs_info, rhs_info);
        (NodeInfo::new(Some(data_type), None), vec![
            position.convert(Node::BinaryOperation {
                lhs: Box::new(lhs_ast[0].clone()), // TODO: check if more than 1 node
                op,
                rhs: Box::new(rhs_ast[0].clone()) // TODO: check if more than 1 node
            })
        ])
    }

    fn check_member_access(&mut self, position: Positioned<()>, lhs: Positioned<Node>, op: Positioned<Operator>, rhs: Positioned<Node>) -> (NodeInfo, Vec<Positioned<Node>>) {
        let (lhs_info, lhs_ast) = self.check_node(lhs);

        if let Some(symbol) = lhs_info.symbol {
            let class_symbol = match symbol {
                Symbol::Function(_) => panic!("Access impossible in function"),
                Symbol::Variable(variable) => {
                    if let Some(DataType::Custom(data_type)) = variable.lock().unwrap().data_type.clone() {
                        if let Some(class_symbol) = self.scope.get_class(data_type) {
                            class_symbol
                        } else {
                            panic!("Could not get class!")
                        }
                    } else {
                        panic!("Could not infer type of variable!")
                    }
                },
                Symbol::Class(class) => class,
            };

            // Select the scope
            let mut scope = Box::new(Scope::new(ScopeType::Class(class_symbol.lock().unwrap().name.clone()), None));

            // Push the fields to the selected scope
            for field in class_symbol.lock().unwrap().fields.iter() {
                scope.variables.push(field.clone());
            }

            // Push the functions to the selected scope
            for function in class_symbol.lock().unwrap().functions.iter() {
                scope.functions.push(function.clone());
            }

            self.scope.selected = Some(scope);

            // Process rhs
            let (rhs_info, rhs_ast) = self.check_node(rhs);
            
            (rhs_info, vec![
                position.convert(Node::BinaryOperation { 
                    lhs: Box::new(lhs_ast[0].clone()), 
                    op, 
                    rhs: Box::new(rhs_ast[0].clone()) 
                })
            ])
        } else {
            panic!("Nothing to access (issue with the lhs)");
        }
    }

    fn check_binary_operation(&mut self, position: Positioned<()>, lhs: Positioned<Node>, op: Positioned<Operator>, rhs: Positioned<Node>) -> (NodeInfo, Vec<Positioned<Node>>) {        
        // TODO: Check binary operation

        match op.data {
            Operator::Plus => todo!("Check plus"),
            Operator::Minus => todo!("Check minus"),
            Operator::Multiply => todo!("Check multiply"),
            Operator::Divide => todo!("Check divide"),
            Operator::MemberAccess => self.check_member_access(position, lhs, op, rhs),
            Operator::Assignment => self.check_assignment(position, lhs, op, rhs),
        }
    }

    fn check_variable_definition(&mut self, position: Positioned<()>, var_type: Positioned<VarType>, name: Positioned<String>, data_type: Option<Positioned<String>>, value: Option<Box<Positioned<Node>>>) -> (NodeInfo, Vec<Positioned<Node>>) {
        if self.scope.get_variable(name.data.clone()).is_some() {
            panic!("Shadowing of variable impossible!");
        }

        // Infer and check
        let final_data_type = if let Some(value) = value.clone() {
            let (value_info, _value_ast) = self.check_node(*value);

            if let Some(data_type) = data_type {
                if let Some(value_info_type) = value_info.data_type {
                    // Check if the types match
                    if self.check_data_type(value_info_type, DataType::Custom(data_type.data.clone())) {
                        Some(DataType::Custom(data_type.data.clone()))
                    } else {
                        panic!("Unexpected data_type!")
                    }
                } else {
                    // Type can be inferred for rhs
                    Some(self.infer_and_check(value_info, DataType::Custom(data_type.data.clone())))
                }
            } else {
                if let Some(value_info_type) = value_info.data_type {
                    Some(value_info_type.clone())
                } else {
                    panic!("Cannot infer type (type constraints not available)")
                }
            }
        } else if let Some(data_type) = data_type {
            Some(DataType::Custom(data_type.data.clone()))
        } else {
            None
        };

        // Add Symbol
        self.scope.variables.push(Arc::new(Mutex::new(VariableSymbol::new(var_type.data.clone(), name.data.clone(), final_data_type.clone(), value.is_some()))));

        // Push AST
        (NodeInfo::new(Some(DataType::Void), None), vec![
            position.convert(Node::VariableDefinition { 
                var_type: var_type.clone(), 
                name: name.clone(), 
                data_type: final_data_type.map(|x| position.convert(x.to_string())), // TODO: find a better option than using position (maybe value) 
                value: value.clone() 
            })
        ])
    }

    fn check_function_definition(&mut self, position: Positioned<()>, name: Positioned<String>, return_type: Option<Positioned<String>>, params: Vec<FunctionDefinitionParameter>, body: Option<Vec<Positioned<Node>>>, constructor: bool) -> (NodeInfo, Vec<Positioned<Node>>) {
        if self.scope.get_function(name.data.clone()).is_some() {
            panic!("Shadowing of function impossible!");
        }

        match self.scope.scope {
            ScopeType::Function(_, _) => panic!("Cannot declare function inside of function!"),
            _ => {}
        }

        let data_type = return_type.map_or_else(|| DataType::Void, |x| DataType::Custom(x.data.clone()));
        let function_type = if body.is_none() {
            FunctionType::ExternalFunction
        } else if constructor {
            FunctionType::Constructor
        } else {
            FunctionType::Function
        };

        // Add Symbol
        self.scope.functions.push(Arc::new(Mutex::new(FunctionSymbol::new(name.data.clone(), data_type.clone(), function_type, params.clone()))));
        
        
        // Process body
        let new_body = if let Some(body) = body {
            // Enter Scope
            let parent = std::mem::replace(&mut self.scope, Scope::new(ScopeType::Root, None));
            self.scope = Scope::new(ScopeType::Function(name.data.clone(), data_type.clone()), Some(Box::new(parent)));
            
            // TODO: Push the params in the scope
            for param in params.iter() {
                self.scope.variables.push(Arc::new(Mutex::new(VariableSymbol { 
                    var_type: VarType::Constant, 
                    name: param.name.data.clone(), 
                    data_type: Some(DataType::Custom(param.data_type.data.clone())), 
                    initialized: true 
                })))
            }

            // TODO: Add allocation if constructor

            // Check body
            let mut new_body = Vec::new();
            for node in body {
                let (node_info, mut node_ast) = self.check_node(node.clone());

                // TODO: also check last node's type

                new_body.append(&mut node_ast);
            }

            // Exit scope
            let scope = std::mem::replace(&mut self.scope, Scope::new(ScopeType::Root, None));
            self.scope = *scope.parent.unwrap();
            Some(new_body)
        } else {
            None
        };

        (NodeInfo::new(Some(DataType::Void), None), vec![
            position.convert(Node::FunctionDefinition { name, return_type: Some(position.convert(data_type.to_string())), params, body: new_body, constructor }) // TODO: change the position of data_type
        ])
    }

    fn check_return(&mut self, position: Positioned<()>, value: Positioned<Node>) -> (NodeInfo, Vec<Positioned<Node>>) {
        match self.scope.scope.clone() {
            ScopeType::Function(_, data_type) => {
                let (value_info, value_ast) = self.check_node(value).clone();
                let data_type = self.infer_and_check(value_info, data_type);

                // TODO: check if more than 1 value (in the ast)
                (NodeInfo::new(Some(DataType::Void), None), vec![position.convert(Node::Return(Box::new(value_ast[0].clone())))])
            },
            _ => panic!("Unexpected return statement!")
        }
    }

    fn check_function_call(&mut self, position: Positioned<()>, name: Positioned<String>, params: Vec<FunctionCallParameter>) -> (NodeInfo, Vec<Positioned<Node>>) {
        if let Some(function_arc) = self.scope.get_function(name.data.clone()) {
            let function = function_arc.lock().unwrap();
            let mut new_params = Vec::new();
            let mut index = 0;
            for param in function.params.iter() {
                if let Some(given_param) = params.get(index) {
                    let (param_info, param_ast) = self.check_node(given_param.value.clone());

                    let data_type = self.infer_and_check(param_info, DataType::Custom(param.data_type.data.clone()));

                    // TODO: check if more than 1 value (in the ast)
                    new_params.push(FunctionCallParameter { value: param_ast[0].clone() });
                } else {
                    panic!("Not enough params");
                }
                index += 1;
            }

            if params.len() > index {
                panic!("Too many params");
            }

            (NodeInfo::new(Some(function.data_type.clone()), None), vec![
                position.convert(Node::FunctionCall { name: name.clone(), params: new_params })
            ])
        } else {
            panic!("Function '{}' not found!", name.data);
        }
    }

    fn check_include(&mut self, position: Positioned<()>, path: Positioned<String>) -> (NodeInfo, Vec<Positioned<Node>>) {
        // TODO: find a way to check if the path is valid and exists.
        self.includes.push((position, path));
        (NodeInfo::new(Some(DataType::Void), None), vec![
            // No need to add the node (post processed)
        ])
    }

    fn check_class_definition(&mut self, position: Positioned<()>, name: Positioned<String>, body: Vec<Positioned<Node>>) -> (NodeInfo, Vec<Positioned<Node>>) {
        // check if class doesn't exists
        if self.scope.symbol_exists(name.data.clone()) {
            panic!("Symbol already exists");
        }
        
        // Add Symbol
        let class = Arc::new(Mutex::new(ClassSymbol::new(name.data.clone())));
        self.scope.classes.push(class.clone());

        // Enter scope
        let mut scope = std::mem::replace(&mut self.scope, Scope::new(ScopeType::Root, None));
        self.scope = Scope::new(ScopeType::Class(name.data.clone()), Some(Box::new(scope)));
        
        // Add Self
        self.scope.variables.push(Arc::new(Mutex::new(VariableSymbol::new(VarType::Constant, "self".to_string(), Some(DataType::Custom(name.data.clone())), true))));

        // Check the body
        let mut new_body = Vec::new();
        for node in body {
            match &node.data {
                Node::VariableDefinition { var_type, name, data_type, value } => {
                    let (_, mut ast) = self.check_node(node);
                    new_body.append(&mut ast);
                    // Add variable to symbol (last symbol)
                    let field_symbol = self.scope.variables.last().clone().cloned().unwrap();
                    class.lock().unwrap().fields.push(field_symbol);
                    // self.scope.parent.as_mut().unwrap().classes.last_mut().unwrap().lock().unwrap().fields.push(field_symbol);
                    // TODO: process default value (to be in constructor) [not supported for now]
                },
                Node::FunctionDefinition { name, return_type, params, body, constructor } => {
                    let (_, mut ast) = self.check_node(node);
                    new_body.append(&mut ast);
                    // Add function to symbol (last symbol)
                    let function_symbol = self.scope.functions.last().clone().cloned().unwrap();
                    class.lock().unwrap().functions.push(function_symbol);
                    // self.scope.parent.as_mut().unwrap().classes.last_mut().unwrap().lock().unwrap().functions.push(function_symbol);
                }
                _ => panic!("Unexpected node")
            }
        }

        // Exit the scope
        scope = std::mem::replace(&mut self.scope, Scope::new(ScopeType::Root, None));
        self.scope = *scope.parent.unwrap();

        (NodeInfo::new(Some(DataType::Void), None), vec![
            position.convert(Node::ClassDefinition { name: name.clone(), body: new_body })
        ])
    }

    // Should return the generated AST from the node + the info
    fn check_node(&mut self, node: Positioned<Node>) -> (NodeInfo, Vec<Positioned<Node>>) {
        match node.data.clone() {
            Node::Value(value) => 
                self.check_value(node.convert(value)),
            Node::BinaryOperation { lhs, op, rhs } => 
                self.check_binary_operation(node.convert(()), *lhs, op, *rhs),
            Node::VariableDefinition { var_type, name, data_type, value } => 
                self.check_variable_definition(node.convert(()), var_type, name, data_type, value),
            Node::FunctionDefinition { name, return_type, params, body, constructor } => 
                self.check_function_definition(node.convert(()), name, return_type, params, body, constructor),
            Node::Return(value) => 
                self.check_return(node.convert(()), *value),
            Node::FunctionCall { name, params } => 
                self.check_function_call(node.convert(()), name, params),
            Node::Include(path) => 
                self.check_include(node.convert(()), path),
            Node::ClassDefinition { name, body } => 
                self.check_class_definition(node.convert(()), name, body)
        }
    }

    pub fn check(&mut self) -> Vec<Positioned<Node>> {
        let mut ast = Vec::new();
        while let Some(current) = self.current() {
            let (_, mut ast_res) = self.check_node(current);
            ast.append(&mut ast_res);
            self.advance()
        }
        
        let mut includes = Vec::new();
        for (position, path) in self.includes.iter() {
            includes.push(position.convert(Node::Include(path.clone())));
        }
        includes.append(&mut ast);
        
        includes
    }

}