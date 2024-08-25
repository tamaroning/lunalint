use std::collections::HashMap;
use std::sync::Arc;

use full_moon::visitors::Visit;
use full_moon::{ast, node::Node, visitors::Visitor};
use parking_lot::Mutex;

use crate::location::{Location, SourceInfo};
use crate::utils;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NodeId {
    private: (usize, usize),
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
        self.loc.clone()
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

pub type Scope = Arc<Mutex<HashMap<String, NodeId>>>;

#[derive(Debug)]
pub struct Resolver {
    /// Use-Def relations
    use_defs: HashMap<NodeId, NodeId>,
    /// Def-Use relations
    def_uses: HashMap<NodeId, Vec<NodeId>>,

    /// Assignment relations
    /// e.g. foo = 1; foo = 2
    /// relation: foo (second occurence) -> foo (first occurence)
    reassignments: HashMap<NodeId, NodeId>,

    /// Block to scope mapping
    block_to_scope: HashMap<NodeId, Vec<Scope>>,

    // current lexical scope. After resolving, the first scope only remains
    scopes: Vec<Scope>,
    definitions: HashMap<NodeId, Definition>,
    src: Arc<SourceInfo>,
}

/// Name resolution is splited into two steps:
/// 1. Get all definitions (local variables, global variables, functions)
/// 2. Resolve uses of variables
impl Resolver {
    pub fn new(src: Arc<SourceInfo>) -> Self {
        Resolver {
            use_defs: HashMap::new(),
            def_uses: HashMap::new(),
            reassignments: HashMap::new(),
            block_to_scope: HashMap::new(),
            scopes: Vec::new(),
            definitions: HashMap::new(),
            src,
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
        // Check use-def relation
        if let Some(def_id) = self.use_defs.get(&node_id) {
            return Some(*def_id);
        }
        // Check reassignment relation
        if let Some(def_id) = self.reassignments.get(&node_id) {
            return Some(*def_id);
        }
        None
    }

    pub fn get_definition(&self, def_id: NodeId) -> Option<&Definition> {
        self.definitions.get(&def_id)
    }

    pub fn lookup_scope(&self, block: NodeId) -> Option<&Vec<Scope>> {
        self.block_to_scope.get(&block)
    }

    fn lookup_name(&self, name: &str) -> Option<NodeId> {
        for scope in self.scopes.iter().rev() {
            let scope = scope.lock();
            if let Some(node_id) = scope.get(name) {
                return Some(*node_id);
            }
        }
        None
    }

    fn push_scope(&mut self) {
        self.scopes.push(Arc::new(Mutex::new(HashMap::new())));
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_local_definiton(&mut self, name: String, node_id: NodeId, def: Definition) {
        let last_scope = self.scopes.last_mut().unwrap();
        let mut last_scope = last_scope.lock();
        last_scope.insert(name, node_id);
        self.definitions.insert(node_id, def);
    }

    fn insert_global_definition(&mut self, name: String, node_id: NodeId, def: Definition) {
        let first_scope = self.scopes.first_mut().unwrap();
        let mut first_scope = first_scope.lock();
        first_scope.insert(name, node_id);
        self.definitions.insert(node_id, def);
    }
}

impl<'a> Visitor for Resolver {
    fn visit_ast(&mut self, ast: &ast::Ast) {
        // push the first lexical scope
        self.push_scope();
        for name in utils::builtin_names() {
            let def = Definition::new(Visibility::Global, name.to_owned(), Location::dummy());
            let node_id = NodeId::from(ast);
            self.insert_global_definition(name.to_owned(), node_id, def);
        }

        ast.nodes().visit(self);
        ast.eof().visit(self);
    }

    fn visit_block(&mut self, _node: &ast::Block) {
        self.push_scope();
    }

    fn visit_block_end(&mut self, node: &ast::Block) {
        let node_id = NodeId::from(node);
        self.block_to_scope.insert(node_id, self.scopes.clone());
        self.pop_scope();
    }

    // local functions (e.g. `local function foo() end`)
    fn visit_local_function(&mut self, node: &ast::LocalFunction) {
        let node_id = NodeId::from(node);
        let name = node.name().to_string();
        let loc = Location::from((&self.src, node.name()));
        let def = Definition::new(Visibility::Local, name.clone(), loc);
        log::trace!("insert local function definition {}", name);
        self.insert_local_definiton(name, node_id, def);
    }

    fn visit_function_declaration(&mut self, node: &ast::FunctionDeclaration) {
        let function_name = node.name();

        if function_name.names().len() == 1 {
            let node_id = NodeId::from(function_name);
            let loc = Location::from((&self.src, function_name.names().first().tokens()));
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
            let loc = Location::from((&self.src, name));
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
        // local assignments are always a definiton (including shadowing)
        for name in node.names() {
            let node_id = NodeId::from(name);
            let loc = Location::from((&self.src, name));
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
            let loc = Location::from((&self.src, name));
            let name = utils::ident_as_str(name).to_owned();
            if let Some(def_id) = self.lookup_name(&name) {
                // Found a definition. This is a reassignment.
                log::trace!("found reassignment {}", name);
                self.reassignments.insert(node_id, def_id);
            } else {
                // Definiton not found. This is a global assignment.
                let def = Definition::new(Visibility::Global, name.clone(), loc);
                log::trace!("insert global assignment definition {}", name);
                self.insert_global_definition(name, node_id, def);
            }
        }
    }

    fn visit_expression(&mut self, node: &ast::Expression) {
        if let ast::Expression::Var(var) = node {
            let ast::Var::Name(name) = var else {
                return;
            };
            let name = utils::ident_as_str(name);
            let node_id = NodeId::from(var);
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

    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        for name in node.names() {
            let node_id = NodeId::from(name);
            let loc = Location::from((&self.src, name));
            let name = utils::ident_as_str(name).to_owned();
            let def = Definition::new(Visibility::Local, name.clone(), loc);
            log::trace!("insert loop var decl {}", name);
            self.insert_local_definiton(name, node_id, def);
        }
    }

    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        let name = node.index_variable();
        let node_id = NodeId::from(name);
        let loc = Location::from((&self.src, name));
        let name = utils::ident_as_str(name).to_owned();
        let def = Definition::new(Visibility::Local, name.clone(), loc);
        log::trace!("insert loop var decl {}", name);
        self.insert_local_definiton(name, node_id, def);
    }
}
