use std::sync::Arc;

use crate::resolver::NodeId;
use crate::utils;
use crate::{
    context::Context,
    diagnostics::{self, report},
    impl_lint_pass,
    location::Location,
};
use ariadne::{Label, ReportKind};
use full_moon::{ast, node::Node, visitors::Visitor};

pub(crate) struct UndefinedGlobal {
    ctx: Arc<Context>,
}
impl_lint_pass!("undefined-global", UndefinedGlobal, LintKind::Diagnostics);

impl UndefinedGlobal {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }
}

impl Visitor for UndefinedGlobal {
    fn visit_prefix(&mut self, prefix: &ast::Prefix) {
        let ast::Prefix::Name(name) = prefix else {
            return;
        };
        let name = utils::ident_as_str(name);
        let node_id = NodeId::new(prefix);
        if self.ctx().resolver().lookup_definiton(node_id).is_none() {
            let loc = Location::from(prefix.tokens());
            let mut report = report(
                self,
                ReportKind::Error,
                loc,
                format!("Undefined global `{name}`"),
            );
            if let Some(suggestion) = get_wrong_name_suggestion(self.ctx(), name) {
                if suggestion != name {
                    report = report.with_label(
                        Label::new((self.ctx().file_name(), loc.into()))
                            .with_message(format!("Did you mean `{}`?", suggestion)),
                    );
                } else {
                    report = report.with_label(
                        Label::new((self.ctx().file_name(), loc.into()))
                            .with_message("Similar name not found".to_string()),
                    );
                }
            }

            diagnostics::emit(self, report);
        }
    }
}

fn get_wrong_name_suggestion<'a>(ctx: &'a Context, name: &str) -> Option<&'a str> {
    let mut min_distance = usize::MAX;
    let mut suggestion: Option<&str> = None;
    for found in ctx.resolver().get_first_scope().keys() {
        let distance = levenshtein_distance(name, found);
        if distance < min_distance {
            min_distance = distance;
            suggestion = Some(found);
        }
    }
    suggestion
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut cost: Vec<usize> = (0..=len1).collect();
    let mut new_cost: Vec<usize> = vec![0; len1 + 1];
    for i in 1..=len2 {
        new_cost[0] = i;
        for j in 1..=len1 {
            let cost_replace = cost[j - 1]
                + if s1.chars().nth(j - 1) != s2.chars().nth(i - 1) {
                    1
                } else {
                    0
                };
            let cost_insert = cost[j] + 1;
            let cost_delete = new_cost[j - 1] + 1;
            new_cost[j] = std::cmp::min(std::cmp::min(cost_insert, cost_delete), cost_replace);
        }
        cost.swap_with_slice(&mut new_cost);
    }
    cost[len1]
}
