mod helper;

use std::path::PathBuf;
use strip_ansi_escapes::strip_str;

use insta::Settings;

macro_rules! ident_to_str {
    ($ident:ident) => {
        stringify!($ident).replace("_", "-")
    };
}

macro_rules! lua_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let name = ident_to_str!($name);
            let path = PathBuf::from(format!("tests/lua/{}.lua", name));
            let out = strip_str(helper::run_linter(&path));
            let mut settings = Settings::new();
            settings.set_prepend_module_to_snapshot(false);
            settings.set_omit_expression(true);
            settings.bind(|| insta::assert_snapshot!(out));
        }
    };
}

lua_test!(undefined_global);
lua_test!(parse_error);
