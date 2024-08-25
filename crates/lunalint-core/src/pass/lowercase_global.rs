use std::sync::Arc;

use crate::diagnostics::{emit_report, LintKind, LintLabel, LintLevel, LintReport};
use crate::resolver::{NodeId, Visibility};
use crate::{context::Context, impl_lint_pass};
use full_moon::{ast, visitors::Visitor};

pub struct LowercaseGlobal {
    ctx: Arc<Context>,
}
impl_lint_pass!(
    "lowercase-global",
    LowercaseGlobal,
    LintKind::Diagnostics,
    LintLevel::Error
);

impl LowercaseGlobal {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }
}

fn check_name(pass: &LowercaseGlobal, def_id: NodeId) {
    let Some(def) = pass.ctx().resolver().get_definition(def_id) else {
        return;
    };

    let first_char = def.name().chars().next().unwrap();
    if def.visibility() == Visibility::Global && first_char.is_lowercase() {
        let loc = def.loc();
        let name = def.name();
        emit_report(
            pass,
            LintReport::new(
                pass,
                loc.clone(),
                format!("Global variable `{name}` starts with a lowercase letter"),
            )
            .with_label(LintLabel::new(
                loc,
                "Global variables should start with an uppercase letter".to_string(),
            )),
        );
    }
}

impl Visitor for LowercaseGlobal {
    fn visit_assignment(&mut self, node: &ast::Assignment) {
        for var in node.variables() {
            let ast::Var::Name(_) = var else {
                continue;
            };
            let def_id = NodeId::from(var);
            check_name(self, def_id);
        }
    }
}
