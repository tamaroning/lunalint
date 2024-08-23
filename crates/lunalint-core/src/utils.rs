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

pub(super) fn to_number(e: &Expression) -> Option<f64> {
    match e {
        Expression::Number(n) => match n.token_type() {
            TokenType::Number { text } => parse_lua_number(text.as_str()),
            _ => None,
        },
        _ => None,
    }
}

fn parse_lua_number(e: &str) -> Option<f64> {
    // Convert to lowercase for simplicity
    let e = e.to_ascii_lowercase();

    // Check and strip sign
    let (e, neg) = if e.starts_with('-') {
        (&e[1..], true)
    } else if e.starts_with('+') {
        (&e[1..], false)
    } else {
        (&e[..], false)
    };

    // Check and strip hex, binary, octal prefix
    let (e, radix): (_, u32) = if e.starts_with("0x") {
        (&e[2..], 16)
    } else if e.starts_with("0b") {
        (&e[2..], 2)
    } else if e.starts_with("0o") {
        (&e[2..], 8)
    } else {
        (e, 10)
    };

    // Retrieve exponent part if any. e.g. p+0, e-9
    let (e, mantissa) = if let Some(pos) = e.find("p+") {
        let n = &e[pos + 2..];
        let exp = n
            .parse::<u32>()
            .expect("expected u32 after p+ in numerical literal");
        let v = 2_u32.pow(exp) as f64;
        (&e[..pos], Some(v))
    } else if let Some(pos) = e.find("p-") {
        let n = &e[pos + 2..];
        let exp = n
            .parse::<u32>()
            .expect("expected u32 after p- in numerical literal");
        let v = 1. / (2_u32.pow(exp) as f64);
        (&e[..pos], Some(v))
    } else if let Some(pos) = e.find("p") {
        let n = &e[pos + 1..];
        let exp = n
            .parse::<u32>()
            .expect("expected u32 after p+ in numerical literal");
        let v = 2_u32.pow(exp) as f64;
        (&e[..pos], Some(v))
    } else if let Some(pos) = e.find("e+") {
        let n = &e[pos + 2..];
        let exp = n
            .parse::<u32>()
            .expect("expected u32 after e+ in numerical literal");
        let v = 10_u32.pow(exp) as f64;
        (&e[..pos], Some(v))
    } else if let Some(pos) = e.find("e-") {
        let n = &e[pos + 2..];
        let exp = n
            .parse::<u32>()
            .expect("expected u32 after e- in numerical literal");
        let v = 1. / (10_u32.pow(exp) as f64);
        (&e[..pos], Some(v))
    } else if let Some(pos) = e.find('e') {
        if radix != 16 {
            let n = &e[pos + 1..];
            let exp = n
                .parse::<u32>()
                .expect("expected u32 after e+ in numerical literal");
            let v = 10_u32.pow(exp) as f64;
            (&e[..pos], Some(v))
        } else {
            (&e[..], None)
        }
    } else {
        (&e[..], None)
    };

    let parts = e.split('.').collect::<Vec<&str>>();
    let (int, frac) = if parts.len() == 1 {
        (parts.first().unwrap(), None)
    } else {
        (parts.first().unwrap(), parts.last())
    };

    // parse integer part
    let mut v = i64::from_str_radix(int, radix).expect("expected i64 in numerical literal") as f64;

    // parse and add up fractional part
    if let Some(frac) = frac {
        let shift = frac.len() as u32;
        dbg!(&frac);
        let frac = u64::from_str_radix(frac, radix)
            .expect("expected u64 in frac part in numerical literal");
        let frac = frac as f64 / radix.pow(shift) as f64;
        v += frac;
    };

    // multiply with exponent part
    if let Some(mantissa) = mantissa {
        v *= mantissa;
    }
    dbg!(v);

    if neg {
        Some(-v)
    } else {
        Some(v)
    }
}

// test for parse_lua_number
#[test]
fn test_parse_lua_number() {
    fn approx_assert_eq(a: f64, b: f64) {
        dbg!(a, b);
        assert!(b - 0.001 <= a && a <= b + 0.001);
    }
    // rewrite above all by usinig approx_eq
    approx_assert_eq(parse_lua_number("3.0").unwrap(), 3.0);
    approx_assert_eq(parse_lua_number("3.1416").unwrap(), 3.1416);
    approx_assert_eq(parse_lua_number("314.16e-2").unwrap(), 3.1416);
    approx_assert_eq(parse_lua_number("0.31416E1").unwrap(), 3.1416);
    approx_assert_eq(parse_lua_number("34e1").unwrap(), 340.0);
    approx_assert_eq(parse_lua_number("0x0.1E").unwrap(), 0.1171875);
    approx_assert_eq(parse_lua_number("0xA23p-4").unwrap(), 162.1875);
    approx_assert_eq(parse_lua_number("0x1.921FB54442D18P+1").unwrap(), 3.1416);
}
