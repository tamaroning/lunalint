use full_moon::{
    ast::{self, Expression},
    tokenizer::{Symbol, TokenReference, TokenType},
};
use num_bigint::{BigInt, BigUint};
use num_rational::Ratio;
use num_traits::Num;
use num_traits::ToPrimitive;

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

pub(super) fn to_number(e: &Expression) -> Option<Ratio<BigInt>> {
    match e {
        Expression::Number(n) => match n.token_type() {
            TokenType::Number { text } => parse_lua_number(text.as_str()),
            _ => None,
        },
        Expression::UnaryOperator { unop, expression } => {
            if matches!(unop, ast::UnOp::Minus(_)) {
                to_number(expression).map(|n| -n)
            } else {
                None
            }
        }
        _ => None,
    }
}

// FIXME: remove parsing sign as it is a unary operator
fn parse_lua_number(e: &str) -> Option<Ratio<BigInt>> {
    // Convert to lowercase for simplicity
    let e = e.to_ascii_lowercase();

    // Check and strip hex, binary, octal prefix
    let (e, radix): (_, u32) = if e.starts_with("0x") {
        (&e[2..], 16)
    } else if e.starts_with("0b") {
        (&e[2..], 2)
    } else if e.starts_with("0o") {
        (&e[2..], 8)
    } else {
        (&e, 10)
    };

    // Retrieve exponent part if any. e.g. p+0, e-9
    let (e, mantissa) = if let Some(pos) = e.find("p+") {
        let n = &e[pos + 2..];
        let exp = BigUint::from_str_radix(n, radix).unwrap();
        let Some(exp) = exp.to_u32() else {
            return None;
        };
        let v = BigUint::from(2_u32).pow(exp);
        (&e[..pos], Some(Ratio::new(v.into(), BigInt::from(1))))
    } else if let Some(pos) = e.find("p-") {
        let n = &e[pos + 2..];
        let Some(exp) = BigUint::from_str_radix(n, radix).unwrap().to_u32() else {
            return None;
        };
        let v = BigUint::from(2_u32).pow(exp);
        (&e[..pos], Some(Ratio::new(BigInt::from(1), v.into())))
    } else if let Some(pos) = e.find("p") {
        let n = &e[pos + 1..];
        let Some(exp) = BigUint::from_str_radix(n, radix).unwrap().to_u32() else {
            return None;
        };
        let v = BigUint::from(2_u32).pow(exp);
        (&e[..pos], Some(Ratio::new(v.into(), BigInt::from(1))))
    } else if let Some(pos) = e.find("e+") {
        let n = &e[pos + 2..];
        let Some(exp) = BigUint::from_str_radix(n, radix).unwrap().to_u32() else {
            return None;
        };
        let v = BigUint::from(10_u32).pow(exp);
        (&e[..pos], Some(Ratio::new(v.into(), BigInt::from(1))))
    } else if let Some(pos) = e.find("e-") {
        let n = &e[pos + 2..];
        let Some(exp) = BigUint::from_str_radix(n, radix).unwrap().to_u32() else {
            // fractional part is too big
            return None;
        };
        let v = BigUint::from(10_u32).pow(exp);
        (&e[..pos], Some(Ratio::new(BigInt::from(1), v.into())))
    } else if let Some(pos) = e.find('e') {
        if radix != 16 {
            let n = &e[pos + 1..];
            let exp = BigUint::from_str_radix(n, radix).unwrap();
            let Some(exp) = exp.to_u32() else {
                return None;
            };
            let v = BigUint::from(10_u32).pow(exp);
            (&e[..pos], Some(Ratio::new(v.into(), BigInt::from(1))))
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
    let mut v = Ratio::new(BigInt::from_str_radix(int, radix).unwrap(), BigInt::from(1));

    // parse and add up fractional part
    if let Some(frac) = frac {
        let Some(shift) = BigUint::from(frac.len()).to_u32() else {
            // fractional part is too long
            return None;
        };
        let frac = BigInt::from_str_radix(frac, radix)
            .expect("expected u64 in frac part in numerical literal");
        let radix = BigInt::from(radix);
        let frac = Ratio::new(frac, radix.pow(shift));
        v += frac;
    };

    // multiply with exponent part
    if let Some(mantissa) = mantissa {
        v *= &mantissa;
    }

    Some(v)
}

#[test]
fn test_parse_lua_number() {
    fn approx_assert_eq(f: Ratio<BigInt>, expected: f64) {
        let e = Ratio::from_float(expected).unwrap();
        let eps = Ratio::from_float(0.001).unwrap();
        assert!(&e - &eps <= f && f <= e + eps);
    }
    approx_assert_eq(parse_lua_number("1").unwrap(), 1.0);
    approx_assert_eq(parse_lua_number("0x99999").unwrap(), 0x99999 as f64);

    // tests from lua manual
    // See https://www.lua.org/manual/5.4/manual.html
    approx_assert_eq(parse_lua_number("3.0").unwrap(), 3.0);
    approx_assert_eq(parse_lua_number("3.1416").unwrap(), 3.1416);
    approx_assert_eq(parse_lua_number("314.16e-2").unwrap(), 3.1416);
    approx_assert_eq(parse_lua_number("0.31416E1").unwrap(), 3.1416);
    approx_assert_eq(parse_lua_number("34e1").unwrap(), 340.0);
    approx_assert_eq(parse_lua_number("0x0.1E").unwrap(), 0.1171875);
    approx_assert_eq(parse_lua_number("0xA23p-4").unwrap(), 162.1875);
    approx_assert_eq(parse_lua_number("0x1.921FB54442D18P+1").unwrap(), 3.1416);
}
