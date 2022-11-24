use crate::{util::{position::Positioned, file::SourceFile}, parser::node::Node};

pub enum TranslatorError {
    UnexpectedNode(Positioned<Node>, Option<String>)
}

impl TranslatorError {

    pub fn print_error(&self, src: SourceFile) {
        match self {
            TranslatorError::UnexpectedNode(node, should_be) => {
                println!("[Parser]: Unexpected node '{}' at '{}' in '{}'", node.data, node.start, src.name);
                if let Some(should_be) = should_be {
                    println!(", should be '{}'", should_be);
                }
                println!("\n{}", node.arrow_message(&src.src));
            },
        }
    }

}
