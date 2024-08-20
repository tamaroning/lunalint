mod count_down_loop;
mod global_in_nil_env;
mod lowercase_global;
mod undefined_global;
mod unicode_name;

pub use count_down_loop::CountDownLoop;
pub use global_in_nil_env::GlobalInNilEnv;
pub use lowercase_global::LowercaseGlobal;
pub use undefined_global::UndefinedGlobal;
pub use unicode_name::UnicodeName;

use crate::context::Context;
use full_moon::ast;

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    pub fn add_pass(&mut self, pass: Box<dyn Pass>) {
        self.passes.push(pass);
    }

    pub fn run(&mut self, ast: &ast::Ast) {
        for pass in self.passes.iter_mut() {
            pass.run(ast);
        }
    }
}

// Lint pass which traverses the AST
pub trait Pass {
    fn ctx(&self) -> &Context;
    fn name(&self) -> &'static str;
    fn kind(&self) -> LintKind;
    fn run(&mut self, ast: &full_moon::ast::Ast);
}

#[macro_export]
macro_rules! impl_lint_pass {
    ($name:literal, $pass:ty, $kind:expr) => {
        use $crate::pass::LintKind;
        use $crate::pass::Pass;
        impl Pass for $pass {
            fn ctx(&self) -> &$crate::context::Context {
                &self.ctx
            }

            fn name(&self) -> &'static str {
                $name
            }

            fn kind(&self) -> $crate::pass::LintKind {
                $kind
            }

            fn run(&mut self, ast: &full_moon::ast::Ast) {
                self.visit_ast(ast);
            }
        }
    };
}

pub enum LintKind {
    Diagnostics,
    SyntaxError,
}

impl LintKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Diagnostics => "diagnostics",
            Self::SyntaxError => "syntax-errors",
        }
    }
}
