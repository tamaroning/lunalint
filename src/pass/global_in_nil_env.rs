use std::sync::Arc;

use super::utils;
use crate::{
    context::Context,
    diagnostics, impl_lint_pass,
    location::{Location},
};
use ariadne::{Label, Report, ReportKind};
use full_moon::{ast, node::Node, visitors::Visitor};

pub(crate) struct GlobalInNilEnv {
    ctx: Arc<Context>,
}
impl_lint_pass!("global_in_nil_env", GlobalInNilEnv);

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
                diagnostics::emit(
                    &self.ctx,
                    Report::build(ReportKind::Error, self.ctx().file_name(), loc.start())
                        .with_code(999)
                        .with_message("The environment is set to nil".to_string())
                        .with_label(
                            Label::new((self.ctx().file_name(), loc.into()))
                                .with_message("Assignment occurs here".to_string()),
                        )
                        .finish(),
                );
            }
        }
    }
}
