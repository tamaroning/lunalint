use std::sync::Arc;

use super::utils;
use crate::{
    context::Context,
    diagnostics::{self, diag},
    impl_lint_pass,
    location::Location,
};
use ariadne::{Label, ReportKind};
use full_moon::{node::Node, visitors::Visitor};

pub(crate) struct CountDownLoop {
    ctx: Arc<Context>,
}
impl_lint_pass!("count-down-loop", CountDownLoop, LintKind::Diagnostics);

impl CountDownLoop {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }
}

impl Visitor for CountDownLoop {
    fn visit_numeric_for(&mut self, node: &full_moon::ast::NumericFor) {
        let Some(start) = utils::to_integer(node.start()) else {
            return;
        };
        let Some(end) = utils::to_integer(node.end()) else {
            return;
        };
        if node.step().is_some() {
            return;
        }

        if start > end {
            let loc = Location::from(node.start().tokens()) + Location::from(node.end().tokens());
            diagnostics::emit(
                self,
                diag(
                    self,
                    ReportKind::Error,
                    loc,
                    "Count down loop which never reaches end".to_string(),
                )
                .with_label(
                    Label::new((self.ctx().file_name(), loc.into()))
                        .with_message(format!("Do you mean `{}, {}, -1`?", start, end)),
                ),
            );
        }
    }
}
