use std::collections::HashMap;

use full_moon::visitors::Visit;
use full_moon::{ast, node::Node, visitors::Visitor};

use crate::location::Location;
use crate::utils;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NodeId {
    private: (usize, usize),
}

impl NodeId {
    pub fn dummy() -> Self {
        NodeId {
            private: (usize::MAX, usize::MAX),
        }
    }
}

impl std::hash::Hash for NodeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.private.0.hash(state);
        self.private.1.hash(state);
    }
}

impl std::fmt::Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({}-{})", self.private.0, self.private.1)
    }
}

impl NodeId {
    pub fn from(node: &dyn Node) -> Self {
        let range = node.range().unwrap();
        NodeId {
            private: (range.0.bytes(), range.1.bytes()),
        }
    }
}

#[derive(Debug)]
pub struct Definition {
    vis: Visibility,
    name: String,
    loc: Location,
}

impl Definition {
    fn new(vis: Visibility, name: String, loc: Location) -> Self {
        Definition { vis, name, loc }
    }

    pub fn visibility(&self) -> Visibility {
        self.vis
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn loc(&self) -> Location {
        self.loc
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Visibility {
    // Local variables and functions
    Local,
    // Global variables but not functions
    Global,
    // Functions without `local` keyword
    Function,
}

#[derive(Debug)]
pub struct Resolver {
    // Use-Def relations
    use_defs: HashMap<NodeId, NodeId>,
    // Def-Use relations
    def_uses: HashMap<NodeId, Vec<NodeId>>,
    // current lexical scope. After resolving, the first scope only remains
    scopes: Vec<HashMap<String, NodeId>>,
    definitions: HashMap<NodeId, Definition>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            use_defs: HashMap::new(),
            def_uses: HashMap::new(),
            scopes: Vec::new(),
            definitions: HashMap::new(),
        }
    }

    pub fn go(&mut self, ast: &ast::Ast) {
        log::debug!("resolver: start");
        self.visit_ast(ast);
    }

    pub fn lookup_definiton(&self, node_id: NodeId) -> Option<NodeId> {
        if self.definitions.contains_key(&node_id) {
            // This is already a definition
            return Some(node_id);
        }
        self.use_defs.get(&node_id).copied()
    }

    pub fn get_definition(&self, def_id: NodeId) -> Option<&Definition> {
        self.definitions.get(&def_id)
    }

    pub fn get_scopes(&self) -> &Vec<HashMap<String, NodeId>> {
        assert!(self.scopes.len() == 1);
        &self.scopes
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
        let last_scope = self.scopes.last_mut().unwrap();
        last_scope.insert(name, node_id);
        self.definitions.insert(node_id, def);
    }

    fn insert_global_definition(&mut self, name: String, node_id: NodeId, def: Definition) {
        let first_scope = self.scopes.first_mut().unwrap();
        first_scope.insert(name, node_id);
        self.definitions.insert(node_id, def);
    }
}

impl<'a> Visitor for Resolver {
    fn visit_ast(&mut self, ast: &ast::Ast) {
        // push the first lexical scope
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

    // local functions (e.g. `local function foo() end`)
    fn visit_local_function(&mut self, node: &ast::LocalFunction) {
        let node_id = NodeId::from(node);
        let name = node.name().to_string();
        let loc = Location::from(node.name());
        let def = Definition::new(Visibility::Local, name.clone(), loc);
        log::trace!("insert local function definition {}", name);
        self.insert_local_definiton(name, node_id, def);
    }

    fn visit_function_declaration(&mut self, node: &ast::FunctionDeclaration) {
        let function_name = node.name();

        if function_name.names().len() == 1 {
            let node_id = NodeId::from(function_name);
            let loc = Location::from(function_name.names().first().tokens());
            let name = function_name.names().first().unwrap().to_string();
            log::trace!("insert global function definition {}", name);
            let def = Definition::new(Visibility::Function, name.clone(), loc);
            self.insert_global_definition(name, node_id, def)
        }
    }

    fn visit_function_body(&mut self, node: &ast::FunctionBody) {
        self.push_scope();
        // add parameters
        for param in node.parameters() {
            let ast::Parameter::Name(name) = param else {
                continue;
            };
            let node_id = NodeId::from(param);
            let loc = Location::from(name);
            let name = utils::ident_as_str(name).to_owned();
            let def = Definition::new(Visibility::Local, name.clone(), loc);
            log::trace!("insert function parameter {}", name);
            self.insert_local_definiton(name, node_id, def);
        }
    }

    fn visit_function_body_end(&mut self, _node: &ast::FunctionBody) {
        self.pop_scope();
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for name in node.names() {
            let node_id = NodeId::from(name);
            let loc = Location::from(name);
            let name = utils::ident_as_str(name).to_owned();
            let def = Definition::new(Visibility::Local, name.clone(), loc);
            log::trace!("insert local assignment definition {}", name);
            self.insert_local_definiton(name, node_id, def);
        }
    }

    fn visit_assignment(&mut self, node: &ast::Assignment) {
        for var in node.variables() {
            let ast::Var::Name(name) = var else {
                continue;
            };
            let node_id = NodeId::from(var);
            let loc = Location::from(name);
            let name = utils::ident_as_str(name).to_owned();
            if let Some(_) = self.lookup_name(&name) {
                // Found a definition. This is a local assignment.
            } else {
                // Definiton not found. This is a global assignment.
                let def = Definition::new(Visibility::Global, name.clone(), loc);
                log::trace!("insert global assignment definition {}", name);
                self.insert_global_definition(name, node_id, def);
            }
        }
    }

    fn visit_var(&mut self, node: &ast::Var) {
        let ast::Var::Name(name) = node else {
            return;
        };
        let name = utils::ident_as_str(name);
        let node_id = NodeId::from(node);
        if let Some(def_id) = self.lookup_name(name) {
            log::debug!("Resolve use of `{name}`: {:?} -> {:?}", node_id, def_id);
            // Register use-def and def-use relation
            self.use_defs.insert(node_id, def_id);
            self.def_uses
                .entry(def_id)
                .or_insert_with(Vec::new)
                .push(node_id);
        } else {
            // Unresolved name. Error is emitted by undefined-global pass.
            log::debug!("unresolved name: `{}`", name);
        }
    }

    fn visit_prefix(&mut self, prefix: &ast::Prefix) {
        let ast::Prefix::Name(name) = prefix else {
            return;
        };
        let name = utils::ident_as_str(name);
        let node_id = NodeId::from(prefix);
        if let Some(def_node_id) = self.lookup_name(&name) {
            log::debug!(
                "Resolve use of `{name}`: {:?} -> {:?}",
                node_id,
                def_node_id
            );
            // Register use-def and def-use relation
            self.use_defs.insert(node_id, def_node_id);
            self.def_uses
                .entry(def_node_id)
                .or_insert_with(Vec::new)
                .push(node_id);
        } else {
            // Unresolved name. Error is emitted by undefined-global pass.
            log::debug!("unresolved name: `{}`", name);
        }
    }
}
