use std::sync::Arc;

use crate::{
    context::Context,
    diagnostics::{self, report},
    impl_lint_pass,
    location::Location,
};
use ariadne::{Label, ReportKind};
use full_moon::{
    ast,
    tokenizer::{TokenReference, TokenType},
    visitors::Visitor,
};

pub(crate) struct UnicodeName {
    ctx: Arc<Context>,
}
impl_lint_pass!("unicode-name", UnicodeName, LintKind::SyntaxError);

impl UnicodeName {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }
}

fn check_name(pass: &UnicodeName, ident_tok: &TokenReference) {
    let TokenType::Identifier { identifier } = ident_tok.token_type() else {
        unreachable!();
    };
    let name = identifier.as_str();
    let loc = Location::from(ident_tok);
    if !name.chars().all(|c| c.is_ascii()) {
        diagnostics::emit(
            pass,
            report(
                pass,
                ReportKind::Error,
                loc,
                format!("Unicode name `{name}`"),
            )
            .with_label(
                Label::new((pass.ctx().file_name(), loc.into()))
                    .with_message("Only ASCII characters are allowed".to_string()),
            ),
        );
    }
}

impl Visitor for UnicodeName {
    fn visit_var(&mut self, node: &ast::Var) {
        let ast::Var::Name(ident_tok) = node else {
            return;
        };
        check_name(self, ident_tok);
    }

    fn visit_index(&mut self, node: &ast::Index) {
        let ast::Index::Dot {
            dot: _,
            name: ident_tok,
        } = node
        else {
            return;
        };
        check_name(self, ident_tok);
    }

    fn visit_field(&mut self, node: &ast::Field) {
        let ast::Field::NameKey {
            key,
            equal: _,
            value: _,
        } = node
        else {
            return;
        };
        check_name(self, key);
    }

    fn visit_function_name(&mut self, node: &ast::FunctionName) {
        for name in node.names() {
            check_name(self, name);
        }
        if let Some(name) = node.method_name() {
            check_name(self, name);
        }
    }
}
