pub struct SourceFile {
    pub name: String,
    pub src: String
}

impl SourceFile {

    pub fn new(name: String, src: String) -> SourceFile {
        return Self {
            name,
            src
        }
    }

}