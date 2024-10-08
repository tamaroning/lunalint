use std::sync::Arc;

use crate::diagnostics::{emit_report, LintKind, LintLabel, LintLevel, LintReport};
use crate::utils;
use crate::{context::Context, impl_lint_pass, location::Location};
use full_moon::{node::Node, visitors::Visitor};

pub struct CountDownLoop {
    ctx: Arc<Context>,
}
impl_lint_pass!(
    "count-down-loop",
    CountDownLoop,
    LintKind::Diagnostics,
    LintLevel::Error
);

impl CountDownLoop {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }
}

impl Visitor for CountDownLoop {
    fn visit_numeric_for(&mut self, node: &full_moon::ast::NumericFor) {
        let Some(start) = utils::to_number(node.start()) else {
            return;
        };
        let Some(end) = utils::to_number(node.end()) else {
            return;
        };
        if node.step().is_some() {
            return;
        }

        if start > end {
            let loc = Location::from((self.ctx().src(), node.start().tokens()))
                + Location::from((self.ctx().src(), node.end().tokens()));
            // Keep original snippet of the start and end expressions
            emit_report(
                self,
                LintReport::new(
                    self,
                    loc.clone(),
                    "Count down loop which never reaches end".to_string(),
                )
                .with_label(LintLabel::new(
                    loc,
                    format!(
                        "Did you mean `{}, {}, -1`?",
                        node.start().to_string().trim(),
                        node.end().to_string().trim()
                    ),
                )),
            );
        }
    }
}
