use std::sync::Arc;

use crate::resolver::{NodeId, Visibility};
use crate::{
    context::Context,
    diagnostics::{self, report},
    impl_lint_pass,
};
use ariadne::{Label, ReportKind};
use full_moon::{ast, visitors::Visitor};

pub struct LowercaseGlobal {
    ctx: Arc<Context>,
}
impl_lint_pass!("lowercase-global", LowercaseGlobal, LintKind::Diagnostics);

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
    if def.visibility() == Visibility::Global && first_char.is_ascii_lowercase() {
        let loc = def.loc();
        let name = def.name();
        diagnostics::emit(
            pass,
            report(
                pass,
                ReportKind::Error,
                loc,
                format!("Global variable `{name}` starts with a lowercase letter"),
            )
            .with_label(
                Label::new((pass.ctx().file_name(), loc.into()))
                    .with_message("Did you miss `local` or misspell it?".to_string()),
            ),
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

    fn visit_prefix(&mut self, prefix: &ast::Prefix) {
        let ast::Prefix::Name(_) = prefix else {
            return;
        };
        let node_id = NodeId::from(prefix);
        // Ignore unresolved names
        let Some(def_id) = self.ctx().resolver().lookup_definiton(node_id) else {
            return;
        };

        check_name(self, def_id);
    }
}
