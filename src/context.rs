use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

use crate::resolver::Resolver;

pub struct Context {
    input_file: Arc<PathBuf>,
    src: Arc<String>,
    resolver: Resolver,
    saw_error: AtomicBool,
}

impl Context {
    pub fn new(input_file: PathBuf, src: String) -> Self {
        Self {
            input_file: Arc::new(input_file),
            src: Arc::new(src),
            resolver: Resolver::new(),
            saw_error: AtomicBool::new(false),
        }
    }

    pub fn file_name(&self) -> &str {
        self.input_file.file_name().unwrap().to_str().unwrap()
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    pub fn resolver_mut(&mut self) -> &mut Resolver {
        &mut self.resolver
    }

    pub fn saw_error(&self) -> bool {
        self.saw_error.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set_saw_error(&self) {
        self.saw_error
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
