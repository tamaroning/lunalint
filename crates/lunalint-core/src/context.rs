use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

use crate::{diagnostics::LintReport, location::SourceInfo, resolver::Resolver};

pub struct Context {
    resolver: Resolver,
    reports: Mutex<Vec<Arc<LintReport>>>,
    src: Arc<SourceInfo>,
}

impl Context {
    pub fn new(src: Arc<SourceInfo>) -> Self {
        Self {
            resolver: Resolver::new(Arc::clone(&src)),
            reports: Mutex::new(Vec::new()),
            src,
        }
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
        !self.reports.lock().is_empty()
    }

    pub fn push_report(&self, report: LintReport) {
        self.reports.lock().push(Arc::new(report));
    }

    pub fn reports(&self) -> MutexGuard<Vec<Arc<LintReport>>> {
        self.reports.lock()
    }
}
