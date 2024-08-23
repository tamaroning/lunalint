use full_moon::ast;

// build cfg from ast
fn build_cfg(ast: ast::Ast) {
    let builder = Builder;
}

use full_moon::visitors::Visitor;

struct Builder;

trait CfgVisit<V: Visitor> {
    fn walk(&self, v: &mut V);
}

impl<V: Visitor> CfgVisit<V> for ast::Ast {
    fn walk(&self, v: &mut V) {
        self.nodes().walk(v);
    }
}

// ast::Block
impl<V: Visitor> CfgVisit<V> for ast::Block {
    fn walk(&self, v: &mut V) {
        for stmt in self.stmts() {
            stmt.walk(v);
        }
    }
}

// ast::Stmt
impl<V: Visitor> CfgVisit<V> for ast::Stmt {
    fn walk(&self, v: &mut V) {
        match self {
            ast::Stmt::Do(d) => d.walk(v),
            ast::Stmt::Assignment(a) => a.walk(v),
            ast::Stmt::FunctionCall(c) => c.walk(v),
            ast::Stmt::FunctionDeclaration(decl) => decl.walk(v),
            ast::Stmt::GenericFor(f) => f.walk(v),
            ast::Stmt::Goto(g) => todo!(),
            ast::Stmt::Label(l) => todo!(),
            _ => todo!(),
        }
    }
}

// ast::FunctionDeclaration
impl<V: Visitor> CfgVisit<V> for ast::FunctionDeclaration {
    fn walk(&self, v: &mut V) {
        self.body().walk(v);
    }
}

// ast::FunctionBody
impl<V: Visitor> CfgVisit<V> for ast::FunctionBody {
    fn walk(&self, v: &mut V) {
        self.block().walk(v);
    }
}

// ast::Do
impl<V: Visitor> CfgVisit<V> for ast::Do {
    fn walk(&self, v: &mut V) {
        self.block().walk(v);
    }
}

// ast::Assignment
impl<V: Visitor> CfgVisit<V> for ast::Assignment {
    fn walk(&self, v: &mut V) {
        for expr in self.expressions() {
            expr.walk(v);
        }
    }
}

// ast::FunctionCall
impl<V: Visitor> CfgVisit<V> for ast::FunctionCall {
    fn walk(&self, v: &mut V) {
        self.prefix().walk(v);
    }
}

// ast::Prefix
impl<V: Visitor> CfgVisit<V> for ast::Prefix {
    fn walk(&self, v: &mut V) {
        match self {
            &ast::Prefix::Name(_) => {}
            &ast::Prefix::Expression(e) => e.walk(v),
        }
    }
}

// ast::Expression
impl<V: Visitor> CfgVisit<V> for ast::Expression {
    fn walk(&self, v: &mut V) {
        match self {
            _ => {}
        }
    }
}
