use std::sync::Arc;

use crate::diagnostics::{emit_report, LintKind, LintLabel, LintLevel, LintReport};
use crate::resolver::NodeId;
use crate::utils;
use crate::{context::Context, impl_lint_pass, location::Location};
use full_moon::{ast, node::Node, visitors::Visitor};

pub struct UndefinedGlobal {
    ctx: Arc<Context>,
}
impl_lint_pass!(
    "undefined-global",
    UndefinedGlobal,
    LintKind::Diagnostics,
    LintLevel::Error
);

impl UndefinedGlobal {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }
}

fn check_name(pass: &UndefinedGlobal, name: &str, use_: NodeId, loc: Location) {
    if pass.ctx().resolver().lookup_definiton(use_).is_some() {
        return;
    }
    if utils::builtin_names().contains(&name) {
        return;
    }
    let mut report = LintReport::new(pass, loc.clone(), format!("Undefined global `{name}`"));

    if let Some(suggestion) = get_wrong_name_suggestion(pass.ctx(), name) {
        report = report.with_label(LintLabel::new(
            loc,
            format!("Did you mean `{}`?", suggestion),
        ));
    } else {
        report = report.with_label(LintLabel::new(
            loc,
            "Similar name not found in this scope".to_string(),
        ));
    }

    emit_report(pass, report);
}

impl Visitor for UndefinedGlobal {
    fn visit_var(&mut self, node: &ast::Var) {
        let ast::Var::Name(name) = node else {
            return;
        };
        let name = utils::ident_as_str(name);
        // Allow builtin names to be undefined
        if utils::builtin_names().contains(&name) {
            return;
        }

        let node_id = NodeId::from(node);
        let loc = Location::from((self.ctx().src(), node.tokens()));
        check_name(self, name, node_id, loc);
    }

    fn visit_prefix(&mut self, prefix: &ast::Prefix) {
        let ast::Prefix::Name(name) = prefix else {
            return;
        };
        let name = utils::ident_as_str(name);
        // Allow builtin names to be undefined
        if utils::builtin_names().contains(&name) {
            return;
        }

        let node_id = NodeId::from(prefix);
        let loc = Location::from((self.ctx().src(), prefix.tokens()));
        check_name(self, name, node_id, loc);
    }
}

fn get_wrong_name_suggestion<'a>(ctx: &'a Context, name: &str) -> Option<&'a str> {
    let mut min_distance = usize::MAX;
    let mut suggestion: Option<&str> = None;
    let mut found_names: Vec<&str> = Vec::new();
    for scope in ctx.resolver().get_scopes() {
        for found in scope.keys() {
            found_names.push(found);
        }
    }
    found_names.extend(utils::builtin_names());

    for found in found_names {
        if found == name {
            // Use before definiton
            continue;
        }
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
