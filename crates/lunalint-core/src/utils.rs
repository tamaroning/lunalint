use full_moon::{
    ast::{self, Expression},
    tokenizer::{Symbol, TokenReference, TokenType},
};

// Basic functions
// https://www.lua.org/manual/5.4/manual.html#6.1
// https://github.com/LuaLS/lua-language-server/blob/ba8f90eb0fab18ce8aee2bdbf7007dc63050381d/doc/en-us/config.md?plain=1#L1714
const BUILTIN_FUNCTIONS: [&str; 24] = [
    "assert",
    "collectgarbage",
    "dofile",
    "error",
    "getmetatable",
    "ipairs",
    "load",
    "loadfile",
    "next",
    "pairs",
    "pcall",
    "print",
    "rawequal",
    "rawget",
    "rawlen",
    "rawset",
    "select",
    "setmetatable",
    "tonumber",
    "tostring",
    "type",
    "warn",
    "xpcall",
    // FIXME: should be here?
    "require",
];

const BUILTIN_VARIABLES: [&str; 4] = [
    "_VERSION", // FIXME: _ENV should be here?
    "_ENV", "_G", "arg",
];

const STDLIB_MODULES: [&str; 9] = [
    "coroutine",
    "debug",
    "io",
    "math",
    "os",
    "package",
    "string",
    "table",
    "utf8",
];

pub fn builtin_names() -> Vec<&'static str> {
    let mut names = Vec::new();
    names.extend_from_slice(&BUILTIN_FUNCTIONS);
    names.extend_from_slice(&BUILTIN_VARIABLES);
    names.extend_from_slice(&STDLIB_MODULES);
    names
}

pub(super) fn ident_as_str(token: &TokenReference) -> &str {
    match token.token_type() {
        TokenType::Identifier { identifier } => identifier.as_str(),
        _ => unreachable!(),
    }
}

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
