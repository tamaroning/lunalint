

use full_moon::{
    ast::{self, Expression},
    tokenizer::{Symbol, TokenType},
};

pub(super) fn variable_name(var: &ast::Var) -> Option<&str> {
    match var {
        ast::Var::Name(n) => {
            if let TokenType::Identifier { identifier } = n.token_type() {
                Some(identifier.as_str())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) fn is_nil(e: &ast::Expression) -> bool {
    match e {
        Expression::Symbol(s) => {
            matches!(
                s.token_type(),
                TokenType::Symbol {
                    symbol: Symbol::Nil
                }
            )
        }
        _ => false,
    }
}

pub(super) fn to_integer(e: &Expression) -> Option<i64> {
    match e {
        Expression::Number(n) => match n.token_type() {
            TokenType::Number { text } => {
                if text.starts_with("0x") {
                    i64::from_str_radix(&text[2..], 16).ok()
                } else if text.starts_with("0b") {
                    i64::from_str_radix(&text[2..], 2).ok()
                } else if text.starts_with("0o") {
                    i64::from_str_radix(&text[2..], 8).ok()
                } else {
                    text.parse::<i64>().ok()
                }
            }
            _ => None,
        },
        _ => None,
    }
}
