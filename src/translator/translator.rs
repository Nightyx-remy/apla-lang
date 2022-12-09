use crate::{util::{file::SourceFile, position::Positioned}, parser::node::{Node, ValueNode, Operator, VarType, FunctionCallParameter}};

pub struct CFile {
    pub name: String,
    pub header: String,
    pub src: String
}

impl CFile {

    pub fn new(name: String) -> Self {
        Self {
            name,
            header: String::new(),
            src: String::new()
        }
    }

}

pub struct CProject {
    pub files: Vec<CFile>
}

impl CProject {

    pub fn new() -> Self {
        Self {
            files: Vec::new()
        }
    }

    pub fn merge(&mut self, mut file: CFile, current: String) {
        if file.name.is_empty() {
            file.name = current;
        } 

        for f in self.files.iter_mut() {
            if f.name == file.name {
                f.header.push_str(&file.header);
                f.src.push_str(&file.src);
                return;
            }
        }

        self.files.push(file);
    }

    pub fn post_process(&mut self) {
        for file in self.files.iter_mut() {
            if !file.header.is_empty() && !file.src.is_empty() {
                file.src = format!("#include \"{}.h\"\n{}", file.name, file.src);
            }
        }
    }
}

pub struct Translator {
    src: SourceFile,
    ast: Vec<Positioned<Node>>,
    index: usize
}

impl Translator {

    pub fn new(src: SourceFile, ast: Vec<Positioned<Node>>) -> Self {
        Self {
            src,
            ast,
            index: 0
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

    fn translate_type(&mut self, data_type: Option<Positioned<String>>) -> String {
        if let Some(data_type) = data_type {
            match data_type.data.as_str() {
                "c_byte" => "byte".to_string(),
                "c_short" => "short".to_string(),
                "c_int" => "int".to_string(),
                "c_long" => "long".to_string(),
                "c_float" => "float".to_string(),
                "c_double" => "double".to_string(),
                _ => data_type.data.clone()
            }
        } else {
            "void".to_string()
        }
    }

    fn translate_value_node(&mut self, value: Positioned<ValueNode>) -> String {
        match value.data {
            ValueNode::Decimal(x) => x.clone(),
            ValueNode::String(x) => format!("\"{}\"", x),
            ValueNode::VariableCall(x) => x.clone(),
            ValueNode::This => "self".to_string(),
        }
    }

    fn translate_binary_op(&mut self, lhs: Positioned<Node>, op: Positioned<Operator>, rhs: Positioned<Node>) -> String {
        let mut str = String::new();
        
        str.push('(');
        str.push_str(self.translate_node(lhs).as_str());

        match op.data {
            Operator::Plus => str.push_str(" + "),
            Operator::Minus => str.push_str(" - "),
            Operator::Multiply => str.push_str(" * "),
            Operator::Divide => str.push_str(" / "),
            Operator::MemberAccess => str.push_str("->"),
            Operator::Assignment => str.push_str(" = "),
        }

        str.push_str(&self.translate_node(rhs));
        str.push(')');

        str
    }

    fn translate_variable_definition(&mut self, var_type: Positioned<VarType>, name: Positioned<String>, data_type: Option<Positioned<String>>, value: Option<Box<Positioned<Node>>>) -> String {
        let mut str = String::new();
        
        _ = var_type; // Vartype is ignored

        if let Some(data_type) = data_type {
            str.push_str(&self.translate_type(Some(data_type)));
        } else {
            panic!("Missing data_type");
        }

        str.push(' ');
        str.push_str(&name.data);

        if let Some(value) = value {
            str.push_str(" = ");
            str.push_str(&self.translate_node(*value));
        }

        str
    }

    fn translate_return(&mut self, node: Positioned<Node>) -> String {
        let mut str = String::new();

        str.push_str("return ");
        str.push_str(&self.translate_node(node));
        
        str
    }

    fn translate_function_call(&mut self, name: Positioned<String>, params: Vec<FunctionCallParameter>) -> String {
        let mut str = String::new();

        str.push_str(&name.data);
        str.push('(');
        let mut index = 0;
        for param in params {
            if index != 0 {
                str.push_str(", ");
            }
            str.push_str(&self.translate_node(param.value));
            index += 1;
        }
        str.push(')');

        str
    }

    fn translate_node(&mut self, node: Positioned<Node>) -> String {
        match node.data.clone() {
            Node::Value(value) => self.translate_value_node(node.convert(value)),
            Node::BinaryOperation { lhs, op, rhs } => self.translate_binary_op(*lhs, op, *rhs),
            Node::VariableDefinition { var_type, name, data_type, value } => self.translate_variable_definition(var_type, name, data_type, value),
            Node::Return(value) => self.translate_return(*value),
            Node::FunctionCall { name, params } => self.translate_function_call(name, params),
            _ => panic!("Unexpected node {}!", node.data)
        }
    }

    fn translate_root(&mut self, root: Positioned<Node>) -> CFile {
        match root.data {
            Node::VariableDefinition { .. } => {
                todo!("Should variable definition be allowed as root (constant?)")
            },
            Node::FunctionDefinition { name, return_type, params, body, constructor } => {
                let mut file = CFile::new("".to_string());

                if constructor {
                    panic!("Unexpected constructor!");
                }

                if body.is_none() {
                    return file; // Nothing to do
                }

                let mut fun_header = String::new();
                fun_header.push_str(&self.translate_type(return_type));
                fun_header.push(' ');
                fun_header.push_str(&name.data);
                fun_header.push('(');
                let mut index = 0;
                for param in params {
                    if index != 0 {
                        fun_header.push_str(", ");
                    }
                    fun_header.push_str(&self.translate_type(Some(param.data_type)));
                    fun_header.push(' ');
                    fun_header.push_str(&param.name.data);
                    index += 1;
                }
                fun_header.push(')');

            
                if name.data != "main" {
                    // in the .h => type name(params, ...);
                    file.header.push_str(&fun_header);
                    file.header.push_str(";\n");
                }


                // in the .c => type name (params, ...) { body }
                file.src.push_str(&fun_header);
                file.src.push_str(" { ");
                index = 0;
                for node in body.unwrap() {
                    if index == 0 {
                        file.src.push_str("\n");
                    }
                    for line in self.translate_node(node).lines() {
                        file.src.push('\t');
                        file.src.push_str(line);
                        file.src.push(';');
                        file.src.push_str("\n");
                    }
                    index += 1;
                }
                file.src.push_str("}\n");

                file
            },
            Node::Include(path) => {
                let mut file = CFile::new("".to_string());

                if path.data.starts_with("std-") {
                    file.src.push_str("#include <");
                    file.src.push_str(&path.data[4..]);
                    file.src.push_str(".h>");
                } else {
                    file.src.push_str("#include \"");
                    file.src.push_str(&path.data);
                    file.src.push_str(".h\"");
                }
                file.src.push('\n');

                file
            },
            Node::ClassDefinition { name, body } => {
                let mut file = CFile::new(name.data.clone());

                let mut struct_str = String::new();

                struct_str.push_str("typedef struct ");
                struct_str.push_str(&name.data);
                struct_str.push_str("T {");

                let mut field_index = 0;
                for node in body {
                    match node.data {
                        Node::VariableDefinition { var_type, name, data_type, value } => {
                            if field_index == 0 {
                                struct_str.push('\n');
                            }
                            struct_str.push('\t');

                            _ = var_type; // Ignored

                            if let Some(data_type) = data_type {
                                struct_str.push_str(&self.translate_type(Some(data_type)));
                            } else {
                                panic!("Missing data_type");
                            }

                            struct_str.push(' ');
                            struct_str.push_str(&name.data);

                            if let Some(_) = value {
                                todo!("Class Field Default values");
                            }

                            struct_str.push_str(";\n");
                            field_index += 1;
                        },
                        Node::FunctionDefinition { name: function_name, return_type, params, body, constructor } => {
                            if body.is_none() {
                                panic!("Class function shouldn't be external!");
                            }

                            let mut fun_header = String::new();
                            if constructor {
                                fun_header.push_str(&name.data);
                                fun_header.push('*');
                            } else {
                                fun_header.push_str(&self.translate_type(return_type));
                            }
                            fun_header.push(' ');
                            fun_header.push_str(&function_name.data);
                            fun_header.push('(');
                            let mut index = 0;
                            if !constructor {
                                // Push first default param (self)
                                fun_header.push_str(&name.data);
                                fun_header.push_str("* self");
                            }
                            // Normal params
                            for param in params {
                                if index != 0 {
                                    fun_header.push_str(", ");
                                }
                                fun_header.push_str(&self.translate_type(Some(param.data_type)));
                                fun_header.push(' ');
                                fun_header.push_str(&param.name.data);
                                index += 1;
                            }
                            fun_header.push(')');
                        
                            // in the .h => type name(params, ...);
                            file.header.push_str(&fun_header);
                            file.header.push_str(";\n");

                            // in the .c => type name (params, ...) { body }
                            file.src.push_str(&fun_header);
                            file.src.push_str(" { ");
                            let mut index = 0;
                            if constructor {
                                // Allocate memory TODO: optimize in checker (later!)
                                file.src.push_str("\n\t");
                                file.src.push_str(&name.data);
                                file.src.push_str("* self = malloc(sizeof(");
                                file.src.push_str(&name.data);
                                file.src.push_str("));");
                            }
                            for node in body.unwrap() {
                                if index == 0 {
                                    file.src.push_str("\n");
                                }
                                for line in self.translate_node(node).lines() {
                                    file.src.push('\t');
                                    file.src.push_str(line);
                                    file.src.push(';');
                                    file.src.push_str("\n");
                                }
                                index += 1;
                            }
                            file.src.push_str("}\n");
                        },
                        _ => {}
                    }
                }

                struct_str.push_str("} ");
                struct_str.push_str(&name.data);
                struct_str.push_str(";");

                file.header = format!("{}\n{}", struct_str, file.header);

                file
            }
            _ => panic!("Unexpected node!")
        }
    }

    pub fn translate(&mut self) -> CProject {
        let mut project = CProject::new();

        while let Some(current) = self.current() {
            let file = self.translate_root(current);

            project.merge(file, self.src.name.clone());

            self.advance();
        }

        project.post_process();

        project
    }

}