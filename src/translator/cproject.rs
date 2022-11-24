use crate::translator::cnode::CNode;

pub struct CProject {
    pub files: Vec<CFile>   
}

impl CProject {

    pub fn new() -> Self {
        Self {
            files: Vec::new()
        }
    }

}

pub struct CFile {
    pub name: String,
    pub header_ast: Vec<CNode>,
    pub source_ast: Vec<CNode>
}