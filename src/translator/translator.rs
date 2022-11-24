use crate::{util::file::SourceFile, parser::node::Node, translator::{cproject::CProject, error::TranslatorError}};

pub struct Translator {
    src: SourceFile,
    ast: Vec<Node>,
    index: usize
}

impl Translator {

    pub fn new(src: SourceFile, ast: Vec<Node>) -> Self {
        Self {
            src,
            ast,
            index: 0,
        }
    }

    pub fn take(self) -> SourceFile {
        self.src
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current(&self) -> Option<Node> {
        return self.ast.get(self.index).cloned();
    }

    pub fn translate(&mut self) -> Result<CProject, TranslatorError> {

        todo!()
    }

}