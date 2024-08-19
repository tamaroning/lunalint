use std::collections::HashMap;

use full_moon::{ast, node::Node, visitors::Visitor};

struct Resolver<'a> {
    set: HashMap<&'a ast::Var, ()>,
}

impl<'a> Visitor for Resolver<'a> {
    fn visit_function_body(&mut self,_node: &ast::FunctionBody) {
        // push scope
    }

    fn visit_function_body_end(&mut self, node: &ast::FunctionBody) {
        // pop scope
    }

    fn visit_var(&mut self, node: &ast::Var) {

    }
}
