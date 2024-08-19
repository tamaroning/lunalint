use std::collections::HashMap;

use full_moon::visitors::Visit;
use full_moon::{ast, node::Node, tokenizer::Position, visitors::Visitor};

use crate::location::Location;
use crate::utils;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NodeId {
    private: (Position, Position),
}

impl std::hash::Hash for NodeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.private.0.bytes().hash(state);
        self.private.1.bytes().hash(state);
    }
}

impl std::fmt::Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NodeId({}:{}-{}:{})",
            self.private.0.line(),
            self.private.0.character(),
            self.private.1.line(),
            self.private.1.character()
        )
    }
}

impl NodeId {
    pub fn new(node: &dyn Node) -> Self {
        NodeId {
            private: node.range().unwrap(),
        }
    }

    pub fn loc(&self) -> Location {
        Location::new(self.private.0.bytes(), self.private.1.bytes())
    }
}

#[derive(Debug)]
pub struct Definition {
    vis: Visibility,
    name: String,
}

impl Definition {
    fn new(vis: Visibility, name: String) -> Self {
        Definition { vis, name }
    }
}

#[derive(Debug)]
pub enum Visibility {
    Local,
    Global,
}

#[derive(Debug)]
pub struct Resolver {
    // Use-Def relations
    use_defs: HashMap<NodeId, NodeId>,
    // current lexical scope (stack)
    scopes: Vec<HashMap<String, NodeId>>,
    definitions: HashMap<NodeId, Definition>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            use_defs: HashMap::new(),
            scopes: Vec::new(),
            definitions: HashMap::new(),
        }
    }

    pub fn go(&mut self, ast: &ast::Ast) {
        log::debug!("resolver: start");
        self.visit_ast(ast);
    }

    pub fn lookup_definiton(&self, node_id: NodeId) -> Option<&NodeId> {
        self.use_defs.get(&node_id)
    }

    pub fn get_definition(&self, def_id: NodeId) -> Option<&Definition> {
        self.definitions.get(&def_id)
    }

    pub fn get_first_scope(&self) -> &HashMap<String, NodeId> {
        assert!(!self.scopes.is_empty());
        self.scopes.first().unwrap()
    }

    fn lookup_name(&self, name: &str) -> Option<NodeId> {
        for scope in self.scopes.iter().rev() {
            if let Some(node_id) = scope.get(name) {
                return Some(*node_id);
            }
        }
        None
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_local_definiton(&mut self, name: String, node_id: NodeId, def: Definition) {
        log::trace!("insert local definition: {}", name);
        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name, node_id);
        self.definitions.insert(node_id, def);
    }

    fn insert_global_definition(&mut self, name: String, node_id: NodeId, def: Definition) {
        log::trace!("insert global definition: {}", name);
        let scope = self.scopes.first_mut().unwrap();
        scope.insert(name, node_id);
        self.definitions.insert(node_id, def);
    }
}

impl<'a> Visitor for Resolver {
    fn visit_ast(&mut self, ast: &ast::Ast) {
        self.push_scope();
        ast.nodes().visit(self);
        ast.eof().visit(self);
    }

    fn visit_do(&mut self, _node: &ast::Do) {
        self.push_scope();
    }

    fn visit_do_end(&mut self, _node: &ast::Do) {
        self.pop_scope();
    }

    fn visit_function_body(&mut self, _node: &ast::FunctionBody) {
        self.push_scope();
    }

    fn visit_function_body_end(&mut self, _node: &ast::FunctionBody) {
        self.pop_scope();
    }

    // local functions (e.g. `local function foo() end`)
    fn visit_local_function(&mut self, node: &ast::LocalFunction) {
        let node_id = NodeId::new(node);
        let name = node.name().to_string();
        let def = Definition::new(Visibility::Local, name.clone());
        log::trace!("insert local function definition {}", name);
        self.insert_local_definiton(name, node_id, def);
    }

    // global functions
    fn visit_function_name(&mut self, node: &ast::FunctionName) {
        if node.names().len() != 1 {
            return;
        }
        let node_id = NodeId::new(node);
        let name = node.names().first().unwrap().to_string();
        log::trace!("insert global function definition {}", name);
        let def = Definition::new(Visibility::Global, name.clone());
        self.insert_global_definition(name, node_id, def)
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        let node_id = NodeId::new(node);
        for name in node.names() {
            let name = name.to_string();
            let def = Definition::new(Visibility::Local, name.clone());
            log::trace!("insert local assignment definition {}", name);
            self.insert_local_definiton(name, node_id, def);
        }
    }

    fn visit_assignment(&mut self, node: &ast::Assignment) {
        let node_id = NodeId::new(node);
        for var in node.variables() {
            let ast::Var::Name(name) = var else {
                continue;
            };
            let name = utils::ident_as_str(name).to_owned();
            if let Some(_) = self.lookup_name(&name) {
                // Found a definition. This is a local assignment.
            } else {
                // Definiton not found. This is a global assignment.
                let def = Definition::new(Visibility::Global, name.clone());
                log::trace!("insert global assignment definition {}", name);
                self.insert_global_definition(name, node_id, def);
            }
        }
    }

    fn visit_prefix(&mut self, prefix: &ast::Prefix) {
        let ast::Prefix::Name(name) = prefix else {
            return;
        };
        let name = utils::ident_as_str(name);
        let node_id = NodeId::new(prefix);
        if let Some(def_node_id) = self.lookup_name(&name) {
            log::debug!(
                "Resolve use of `{name}`: {:?} -> {:?}",
                node_id,
                def_node_id
            );
            self.use_defs.insert(node_id, def_node_id);
        } else {
            // Unresolved name. Error is emitted by undefined-global pass.
            log::debug!("unresolved name: {}", name);
        }
    }
}
