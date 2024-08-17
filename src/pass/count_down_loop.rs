use std::sync::Arc;

use super::utils;
use crate::{
    context::Context,
    diagnostics, impl_lint_pass,
    location::{Location},
};
use ariadne::{Label, Report, ReportKind};
use full_moon::{node::Node, visitors::Visitor};

pub(crate) struct CountDownLoop {
    ctx: Arc<Context>,
}
impl_lint_pass!("count_down_loop", CountDownLoop);

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
                &self.ctx,
                Report::build(ReportKind::Error, self.ctx().file_name(), loc.start())
                    .with_code(999)
                    .with_message("Count down loop which never reaches end".to_string())
                    .with_label(
                        Label::new((self.ctx().file_name(), loc.into()))
                            .with_message(format!("Do you mean `{}, {}, -1`?", start, end)),
                    )
                    .finish(),
            );
        }
    }
}
