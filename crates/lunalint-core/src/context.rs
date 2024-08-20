use std::{path::PathBuf, sync::Arc};

use parking_lot::{Mutex, MutexGuard};

use crate::{diagnostics::LintReport, location::SourceInfo, resolver::Resolver};

pub struct Context {
    input_file: Arc<PathBuf>,
    resolver: Resolver,
    reports: Mutex<Vec<Arc<LintReport>>>,
    src: Arc<SourceInfo>,
}

impl Context {
    pub fn new(input_file: PathBuf, src: String) -> Self {
        let src = Arc::new(SourceInfo::new(
            input_file.to_str().unwrap().to_string(),
            src,
        ));
        Self {
            input_file: Arc::new(input_file),
            resolver: Resolver::new(Arc::clone(&src)),
            reports: Mutex::new(Vec::new()),
            src,
        }
    }

    pub fn file_name(&self) -> &str {
        self.input_file.file_name().unwrap().to_str().unwrap()
    }

    pub fn src(&self) -> &Arc<SourceInfo> {
        &self.src
    }

    pub fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    pub fn resolver_mut(&mut self) -> &mut Resolver {
        &mut self.resolver
    }

    pub fn saw_error(&self) -> bool {
        self.reports.lock().is_empty()
    }

    pub fn push_report(&self, report: LintReport) {
        self.reports.lock().push(Arc::new(report));
    }

    pub fn reports(&self) -> MutexGuard<Vec<Arc<LintReport>>> {
        self.reports.lock()
    }
}
