use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{diagnostics::LintReport, resolver::Resolver};

pub struct Context {
    input_file: Arc<PathBuf>,
    src: Arc<String>,
    resolver: Resolver,
    reports: Mutex<Vec<LintReport>>,
}

impl Context {
    pub fn new(input_file: PathBuf, src: String) -> Self {
        Self {
            input_file: Arc::new(input_file),
            src: Arc::new(src),
            resolver: Resolver::new(),
            reports: Mutex::new(Vec::new()),
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
        self.reports.lock().unwrap().is_empty()
    }

    pub fn push_report(&self, report: LintReport) {
        self.reports.lock().unwrap().push(report);
    }
}
