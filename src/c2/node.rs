use crate::{
    base::{DayFunction, DayObject, RustFunction, Args},
    manager::RuntimeManager,
    std_modules::{conversion, iter::to_iter_inner},
};
use std::sync::Arc;

#[derive(Debug)]
pub enum Node {
    Block(Block),
    Data(DayObject),
    RustFunction(ConstRustFn),
    FunctionCall {
        expr: Box<Node>,
        args: Vec<Node>,
    },
    Identifier(IdentifierNode),
    Assignment {
        assignee: Box<Node>,
        value: Box<Node>,
    },
    Declaration {
        value: Box<Node>,
    },
    ConstDeclaration {
        value: Box<Node>,
    },
    While {
        condition: Box<Node>,
        block: Block,
    },
    BranchNode(Vec<BranchNode>),
    Index(IndexNode),
    Ret(Option<Arc<Node>>),
    FunctionDeclaration {
        block: Arc<Block>,
        is_closure: bool,
    },
    For {
        //Either this will be a flexible id or a fixed one
        expr: Box<Node>,
        block: Block,
    },
}

impl<'a: 'v, 'v, 's> Node {
    pub fn execute(&self, manager: &Arc<RuntimeManager>) -> ExpressionResult {
        match self {
            Node::Data(data) => ExpressionResult::Value(data.clone()),
            Node::Identifier(id) => ExpressionResult::Value(id.get_var(&manager)),
            Node::FunctionCall { expr, args } => match &**expr {
                Node::Identifier(id) => match id.get_mut(manager) {
                    DayObject::Function(func) => {
                        let mut ar = Vec::with_capacity(args.len());
                        for a in args {
                            ar.push(a.execute(manager).value())
                        }

                        ExpressionResult::Value(func.call(ar))
                    }
                    DayObject::Iter(handle) => {
                        if let Some(obj) = handle.0.next() {
                            ExpressionResult::Value(obj)
                        } else {
                            ExpressionResult::Value(DayObject::None)
                        }
                    }
                    _ => panic!("Err: The function {:?} does not exist!", id),
                },
                n => {
                    let mut ar = vec![];
                    for a in args {
                        ar.push(a.execute(manager).value())
                    }

                    ExpressionResult::Value(n.execute(manager).value().call(ar.clone()))
                }
            },
            Node::Declaration { value: v } => {
                let value = v.execute(manager).value();
                manager.def_var(value);
                ExpressionResult::Value(DayObject::None)
            }
            Node::ConstDeclaration { value: v } => {
                //NOTE Const declarations are currently only important for the parser
                let value = v.execute(manager).value();
                manager.def_var(value);
                ExpressionResult::Value(DayObject::None)
            }
            Node::Assignment { assignee, value: v } => match &**assignee {
                Node::Identifier(id) => {
                    let value = v.execute(manager).value();
                    id.set_var(manager, value);
                    ExpressionResult::Value(DayObject::None)
                }
                Node::Index(inner) => {
                    *inner.get_mut(&manager) = v.execute(manager).value();
                    ExpressionResult::Value(DayObject::None)
                }
                _ => panic!(),
            },
            Node::BranchNode(branches) => {
                for b in branches {
                    match b {
                        BranchNode::If { condition, block } => {
                            if let DayObject::Bool(true) = condition.execute(manager).value() {
                                return block.execute();
                            }
                        }
                        BranchNode::ElseIf { condition, block } => {
                            if let DayObject::Bool(true) = condition.execute(manager).value() {
                                return block.execute();
                            }
                        }
                        BranchNode::Else { block } => return block.execute(),
                    }
                }

                ExpressionResult::Value(DayObject::None)
            }
            Node::While { condition, block } => {
                while let DayObject::Bool(true) = condition.execute(manager).value() {
                    if let ExpressionResult::Return(res) = block.execute() {
                        return ExpressionResult::Return(res);
                    }
                }

                ExpressionResult::Value(DayObject::None)
            }
            Node::FunctionDeclaration { block, is_closure } => {
                dbg_print!("Defining function");

                let blk = DayObject::Function(DayFunction::RuntimeDef(Arc::clone(block)));

                if !is_closure {
                    manager.def_var(blk.clone())
                }

                dbg_print!("Defined function");

                ExpressionResult::Value(blk)
            }
            Node::Index(inner) => ExpressionResult::Value(inner.get_value(manager)),
            Node::Ret(expr) => {
                if let Some(expr) = expr {
                    ExpressionResult::Return(expr.execute(manager).value())
                } else {
                    ExpressionResult::Return(DayObject::None)
                }
            }
            Node::For { expr, block } => {
                println!("FOR LOOP");
                let mut iter = to_iter_inner(expr.execute(manager).value());

                while let Some(i) = iter.0.next() {
                    block.scope.def_var(i);
                    block.execute();
                    block.scope.clear();
                }

                ExpressionResult::Value(DayObject::None)
            }
            Node::Block(b) => b.execute(),
            Node::RustFunction(r) => ExpressionResult::Value(DayObject::Function(DayFunction::Function(r.clone().0))),
        }
    }

    pub fn function_decl(block: Block, is_closure: bool) -> Self {
        Self::FunctionDeclaration {
            block: Arc::new(block),
            is_closure,
        }
    }
}

#[derive(Debug)]
pub enum BranchNode {
    If { condition: Box<Node>, block: Block },
    ElseIf { condition: Box<Node>, block: Block },
    Else { block: Block },
}

pub struct Block {
    pub block: RootNode,
    pub scope: Arc<RuntimeManager>,
}

impl Block {
    pub fn new(purpose: NodePurpose, scope: Arc<RuntimeManager>) -> Self {
        Self {
            block: RootNode {
                purpose,
                nodes: Default::default(),
            },
            scope,
        }
    }

    pub fn new_capacity(purpose: NodePurpose, inner_capacity: usize) -> Self {
        Self {
            block: RootNode {
                purpose,
                nodes: Default::default(),
            },
            scope: Arc::new(RuntimeManager::new_capacity(inner_capacity)),
        }
    }

    pub fn new_capacity_predecessor(purpose: NodePurpose, inner_capacity: usize, predecessor: Option<Arc<RuntimeManager>>) -> Self {
        Self {
            block: RootNode {
                purpose,
                nodes: Default::default(),
            },
            scope: Arc::new(RuntimeManager::new_capacity_predecessor(inner_capacity, predecessor)),
        }
    }

    pub fn push(&mut self, node: Node) {
        self.block.push(node)
    }

    pub fn pop(&mut self) -> Option<Node> {
        self.block.pop()
    }

    pub fn len(&self) -> usize {
        self.block.len()
    }

    pub fn execute(&self) -> ExpressionResult {
        self.block.execute(&self.scope)
    }

    pub fn execute_args(&self, args: Args) -> ExpressionResult {
        self.scope.def_args_alloc(args);
        self.block.execute(&self.scope)
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        //FIXME Dummy impl
        write!(f, "{:#?}", self.block)
    }
}

#[derive(Debug)]
pub struct RootNode {
    nodes: Vec<Node>,
    pub purpose: NodePurpose,
}

impl<'a: 'v, 'v, 's> RootNode {
    pub fn new(purpose: NodePurpose) -> Self {
        Self {
            nodes: Default::default(),
            purpose,
        }
    }

    pub fn push(&mut self, node: Node) {
        self.nodes.push(node)
    }

    pub fn pop(&mut self) -> Option<Node> {
        self.nodes.pop()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn execute(&'s self, manager: &Arc<RuntimeManager>) -> ExpressionResult {
        for n in self.nodes.iter() {
            match n.execute(manager) {
                ExpressionResult::Return(res) => {
                    return if self.purpose == NodePurpose::Function {
                        ExpressionResult::Value(res)
                    } else {
                        ExpressionResult::Return(res)
                    }
                }
                v => v,
            };
        }

        ExpressionResult::Value(DayObject::None)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum IdentifierNode {
    Identifier {
        id: usize,
        depth: usize,
    },
    Args
}

impl IdentifierNode {
    pub fn get_var(&self, manager: &Arc<RuntimeManager>) -> DayObject {
        match self {
            IdentifierNode::Identifier {id, depth} => manager.get_var(*id, *depth),
            IdentifierNode::Args => todo!(),
        }
    }

    pub fn get_mut(&self, manager: &Arc<RuntimeManager>) -> &mut DayObject {
        match self {
            IdentifierNode::Identifier {id, depth} => manager.get_var_mut(*id, *depth),
            IdentifierNode::Args => todo!(),
        }
    }

    pub fn set_var(&self, manager: &Arc<RuntimeManager>, val: DayObject) {
        match self {
            IdentifierNode::Identifier {id, depth} => manager.set_var(val, *id, *depth),
            IdentifierNode::Args => panic!("Not allowed"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NodePurpose {
    Function,
    Conditional,
    While,
    Block,
    TopLevel,
    For,
}

impl<'a> IntoIterator for RootNode {
    type Item = Node;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

#[derive(Debug)]
pub struct IndexOperation {
    pub index: Box<Node>,
}

#[derive(Debug, Clone)]
pub enum ExpressionResult {
    Return(DayObject),
    Value(DayObject),
    Yielded(DayObject),
}

impl ExpressionResult {
    ///Retrievs the value if this is the value variant
    pub fn value(self) -> DayObject {
        match self {
            Self::Value(d) => d,
            er => panic!("Expected value received {:?}", er),
        }
    }
}

#[derive(Debug)]
pub struct IndexNode {
    pub initial: Box<Node>,
    pub index_ops: Vec<IndexOperation>,
}

impl IndexNode {
    pub fn get_value(&self, manager: &Arc<RuntimeManager>) -> DayObject {
        self.get_mut(manager).clone()
    }

    pub fn get_mut(&self, manager: &Arc<RuntimeManager>) -> &mut DayObject {
        //NOTE get var mut could be unsafe, maybe it should give
        //out an Arc<UnsafeCell> if any UB occurs
        match &*self.initial {
            Node::Identifier(id) => {
                //NOTE get var mut could be unsafe, maybe it should give
                //out an Arc<UnsafeCell> if any UB occurs
                    match id {
                        IdentifierNode::Identifier { id, depth } => {
                            let mut current = manager.get_var_mut(*id, *depth);
                            for i in &self.index_ops {
                                current = match current {
                                    DayObject::Array(a) => {
                                        &mut a[conversion::to_int_inner(
                                            i.index.execute(&manager).value(),
                                        ) as usize]
                                    }
                                    n => panic!("Can't index into {:?}", n),
                                }
                            }
            
                            current
                        }
                        IdentifierNode::Args => {
                            let mut current = &mut manager.get_args_mut()[conversion::to_int_inner(self.index_ops[0].index.execute(manager).value()) as usize];
                            for i in self.index_ops.iter().skip(1) {
                                current = match current {
                                    DayObject::Array(a) => {
                                        &mut a[conversion::to_int_inner(
                                            i.index.execute(&manager).value(),
                                        ) as usize]
                                    }
                                    n => panic!("Can't index into {:?}", n),
                                }
                            }
            
                            current
                        }
                    }
            }
            _ => todo!("currently assigning to an index to a temporary is not allowed (this has to change for references)")
            /* i => {
                let mut current = i.execute(Arc::clone(&var_manager)).value();

                for i in &self.index_ops {
                    current = match current {
                        DayObject::Array(a) => a[conversion::to_int_inner(
                            i.index.execute(Arc::clone(&var_manager)).value(),
                        ) as usize]
                            .clone(),
                        n => panic!("Can't index into {:?}", n),
                    }
                }
                current
            } */
        }
    }
}

use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone)]
pub struct ConstRustFn(pub RustFunction);

impl Debug for ConstRustFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "RustFunction")
    }
}
