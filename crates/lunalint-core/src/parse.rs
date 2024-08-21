use std::sync::Arc;

use full_moon::ast;

use crate::{
    diagnostics::{LintLabel, LintReport},
    location::{Location, Position, SourceInfo},
};

pub fn parse(src: Arc<SourceInfo>) -> Result<full_moon::ast::Ast, LintReport> {
    full_moon::parse(src.content()).map_err(|e| convert_error(e, src))
}

fn convert_error(e: full_moon::Error, src: Arc<SourceInfo>) -> LintReport {
    match e {
        full_moon::Error::TokenizerError(e) => {
            let pos = Position::from_fullmoon_position(e.position());
            let loc = Location::from_position(src, pos);
            let msg = e.error().to_string();
            LintReport::new_parse_error(loc.clone(), "Error occured while tokenizing".to_owned())
                .with_label(LintLabel::new(loc, msg))
        }
        full_moon::Error::AstError(e @ (ast::AstError::Empty | ast::AstError::NoEof)) => {
            LintReport::new_parse_error(Location::dummy(), e.to_string())
        }
        full_moon::Error::AstError(ast::AstError::UnexpectedToken { token, additional }) => {
            let loc = Location::new(
                src,
                Position::from_fullmoon_position(token.start_position()),
                Position::from_fullmoon_position(token.end_position()),
            );
            let msg = format!(
                "Unexpected token `{}`{}",
                token.to_string(),
                additional.map(|s| format!(", {}", s)).unwrap_or_default()
            );
            LintReport::new_parse_error(loc.clone(), "Error occured while parsing".to_owned())
                .with_label(LintLabel::new(loc, msg))
        }
    }
}
