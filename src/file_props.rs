pub struct FileProps {
    line_ending: String
}

impl FileProps {
    pub fn new() -> Self {
        Self {
            line_ending: if cfg!(target_os = "windows") {
                "\r\n".to_string()
            } else {
                "\n".to_string()
            }
        }
            
    }
    pub fn line_endng(&self) -> String {
        self.line_ending.clone()
    }
}
