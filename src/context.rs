use std::{path::PathBuf, sync::Arc};

pub struct Context {
    input_file: Arc<PathBuf>,
    src: Arc<String>,
}

impl Context {
    pub fn new(input_file: PathBuf, src: String) -> Self {
        Self {
            input_file: Arc::new(input_file),
            src: Arc::new(src),
        }
    }

    pub fn file_name(&self) -> &str {
        self.input_file.file_name().unwrap().to_str().unwrap()
    }

    pub fn src(&self) -> &str {
        &self.src
    }
}
