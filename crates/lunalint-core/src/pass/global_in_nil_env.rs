use std::sync::Arc;

use crate::diagnostics::{emit_report, LintKind, LintLabel, LintLevel, LintReport};
use crate::utils;
use crate::{context::Context, impl_lint_pass, location::Location};
use full_moon::{ast, node::Node, visitors::Visitor};

pub struct GlobalInNilEnv {
    ctx: Arc<Context>,
}
impl_lint_pass!(
    "global-in-nil-env",
    GlobalInNilEnv,
    LintKind::Diagnostics,
    LintLevel::Error
);

impl GlobalInNilEnv {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }
}

impl Visitor for GlobalInNilEnv {
    fn visit_assignment(&mut self, node: &ast::Assignment) {
        let vars = node.variables().iter();
        let mut exprs = node.expressions().iter();
        for var in vars {
            let is_nil = if let Some(expr) = exprs.next() {
                utils::is_nil(expr)
            } else {
                true
            };
            if utils::variable_name(var) == Some("_ENV") && is_nil {
                let loc = Location::from(var.tokens());
                emit_report(
                    self,
                    LintReport::new(self, loc, "Invalid global (`_ENV` is `nil`)".to_string())
                        .with_label(LintLabel::new(loc, "Assignment occurs here".to_string())),
                );
            }
        }
    }
}
