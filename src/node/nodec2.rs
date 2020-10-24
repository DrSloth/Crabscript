use crate::std_modules::{conversion, iter::to_iter_inner};
use crate::{
    base::{DayFunction, DayObject, RustFunction},
    variables::Variables,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Node {
    RootNode(RootNode),
    Data(DayObject),
    RustFunction(RustFunction),
    FunctionCall {
        expr: Box<Node>,
        args: Vec<Node>,
    },
    Identifier {
        id: usize,
    },
    Declaration {
        id: usize,
        value: Box<Node>,
    },
    Assignment {
        assignee: Box<Node>,
        value: Box<Node>,
    },
    ConstDeclaration {
        id: usize,
        value: Box<Node>,
    },
    While {
        condition: Box<Node>,
        block: RootNode,
    },
    BranchNode(Vec<BranchNode>),
    Index(IndexNode),
    Ret(Option<Arc<Node>>),
    FunctionDeclaration {
        block: Arc<RootNode>,
        id: Option<u64>,
    },
    For {
        //Either this will be a flexible ident or a fixed one
        id: usize,
        expr: Box<Node>,
        block: RootNode,
    },
}

impl<'a: 'v, 'v, 's> Node {
    pub fn execute(&self, var_manager: &Arc<Variables<'v>>) -> ExpressionResult {
        match self {
            Node::Data(data) => ExpressionResult::Value(data.clone()),
            Node::FunctionCall { expr, args } => match &**expr {
                Node::Identifier { id, hash } => match var_manager.get_var_mut_hash(*hash, id) {
                    DayObject::Function(func) => {
                        let mut ar = Vec::with_capacity(args.len());
                        for a in args {
                            ar.push(a.execute(var_manager).value())
                        }

                        ExpressionResult::Value(func.call(ar, var_manager))
                    }
                    DayObject::Iter(handle) => {
                        if let Some(obj) = handle.0.next(var_manager) {
                            ExpressionResult::Value(obj)
                        } else {
                            ExpressionResult::Value(DayObject::None)
                        }
                    }
                    _ => panic!("Err: The function {} does not exist!", id),
                },
                n => {
                    let mut ar = vec![];
                    for a in args {
                        ar.push(a.execute(var_manager).value())
                    }

                    ExpressionResult::Value(
                        n.execute(var_manager).value().call(ar.clone(), var_manager),
                    )
                }
            },
            Node::Identifier { id, hash } => {
                ExpressionResult::Value(var_manager.get_var_hash(*hash, id))
            }
            Node::Declaration { id, value: v, hash } => {
                let value = v.execute(var_manager).value();
                var_manager.def_var_ref_hash(*hash, id, value);
                ExpressionResult::Value(DayObject::None)
            }
            Node::ConstDeclaration { id, value: v ,hash } => {
                let value = v.execute(var_manager).value();
                Arc::clone(var_manager).def_const_ref_hash(*hash, id, value);
                ExpressionResult::Value(DayObject::None)
            }
            Node::Assignment { assignee, value: v } => match &**assignee {
                Node::Identifier { id, hash } => {
                    let value = v.execute(var_manager).value();
                    var_manager.set_var_ref_hash(*hash, id, value);
                    ExpressionResult::Value(DayObject::None)
                }
                Node::Index(inner) => {
                    *inner.get_mut(Arc::clone(&var_manager)) = v.execute(var_manager).value();
                    ExpressionResult::Value(DayObject::None)
                }
                _ => panic!(),
            },
            Node::BranchNode(branches) => {
                for b in branches {
                    match b {
                        BranchNode::If { condition, block } => {
                            if let DayObject::Bool(true) = condition.execute(&var_manager).value() {
                                return block.execute(&var_manager.get_new_scope());
                            }
                        }
                        BranchNode::ElseIf { condition, block } => {
                            if let DayObject::Bool(true) = condition.execute(var_manager).value() {
                                return block.execute(&var_manager.get_new_scope());
                            }
                        }
                        BranchNode::Else { block } => {
                            return block.execute(&var_manager.get_new_scope())
                        }
                    }
                }

                ExpressionResult::Value(DayObject::None)
            }
            Node::While { condition, block } => {
                while let DayObject::Bool(true) = condition.execute(var_manager).value() {
                    if let ExpressionResult::Return(res) =
                        block.execute(&var_manager.get_new_scope())
                    {
                        return ExpressionResult::Return(res);
                    }
                }

                ExpressionResult::Value(DayObject::None)
            }
            Node::FunctionDeclaration { block, id } => {
                dbg_print!(format!("Defining function {:?}", id));

                let fid = if let Some(id) = id {
                    var_manager.def_fn(id.1, id.0, Arc::clone(&block))
                } else {
                    var_manager.def_closure(Arc::clone(&block))
                };

                dbg_print!(format!("{:?} is now {}", id, fid));

                ExpressionResult::Value(DayObject::Function(DayFunction::RuntimeDef(fid)))
            }
            Node::Index(inner) => ExpressionResult::Value(inner.get_value(Arc::clone(var_manager))),
            Node::Ret(expr) => {
                if let Some(expr) = expr {
                    ExpressionResult::Return(expr.execute(var_manager).value())
                } else {
                    ExpressionResult::Return(DayObject::None)
                }
            }
            Node::For {
                ident,
                expr,
                block,
                hash,
            } => {
                let mut iter = to_iter_inner(expr.execute(var_manager).value(), var_manager);

                let scope = var_manager.get_new_scope();
                if *ident != "_" {
                    while let Some(i) = iter.0.next(&var_manager) {
                        scope.def_var_ref_hash(*hash, ident, i);
                        block.execute(&scope);
                        scope.clear_scope_ref();
                    }
                } else {
                    while let Some(_) = iter.0.next(&var_manager) {
                        block.execute(&scope);
                        scope.clear_scope_ref();
                    }
                }

                ExpressionResult::Value(DayObject::None)
            }
            Node::RootNode(r) => r.execute(var_manager),
        }
    }

    pub fn function_decl(id: Option<&'a str>, block: RootNode) -> Self {
        Self::FunctionDeclaration {
            id: if let Some(id) = id {
                Some((id, crate::variables::hash(id)))
            } else {
                None
            },
            block: Arc::new(block),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BranchNode {
    If {
        condition: Box<Node>,
        block: RootNode,
    },
    ElseIf {
        condition: Box<Node>,
        block: RootNode,
    },
    Else {
        block: RootNode,
    },
}

#[derive(Debug, Clone)]
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

    pub fn execute(&'s self, var_manager: &Arc<Variables<'v>>) -> ExpressionResult {
        for n in self.nodes.iter() {
            match n.execute(var_manager) {
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

#[derive(Debug, Clone)]
pub struct IndexOperation<'a> {
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

#[derive(Debug, Clone)]
pub struct IndexNode {
    pub initial: Box<Node>,
    pub index_ops: Vec<IndexOperation<'a>>,
}

impl<'a: 'v, 'v> IndexNode {
    pub fn get_value(&self, var_manager: Arc<Variables<'v>>) -> DayObject {
        match &*self.initial {
            Node::Identifier { id, hash } => {
                //NOTE get var mut could be unsafe, maybe it should give
                //out an Arc<UnsafeCell> if any UB occurs
                let mut current = var_manager.get_var_mut_hash(*hash, id);
                for i in &self.index_ops {
                    current = match current {
                        DayObject::Array(a) => {
                            &mut a[conversion::to_int_inner(i.index.execute(&var_manager).value())
                                as usize]
                        }
                        n => panic!("Can't index into {:?}", n),
                    }
                }
                current.clone()
            }
            i => {
                let mut current = i.execute(&var_manager).value();

                for i in &self.index_ops {
                    current = match current {
                        DayObject::Array(a) => {
                            a[conversion::to_int_inner(i.index.execute(&var_manager).value())
                                as usize]
                                .clone()
                        }
                        n => panic!("Can't index into {:?}", n),
                    }
                }
                current
            }
        }
    }

    pub fn get_mut(&self, var_manager: Arc<Variables<'v>>) -> &'v mut DayObject {
        match &*self.initial {
            Node::Identifier{id, hash} => {
                //NOTE get var mut could be unsafe, maybe it should give
                //out an Arc<UnsafeCell> if any UB occurs
                let mut current = var_manager.get_var_mut_hash(*hash, id);
                for i in &self.index_ops {
                    current = match current {
                        DayObject::Array(a) => {
                            &mut a[conversion::to_int_inner(
                                i.index.execute(&var_manager).value(),
                            ) as usize]
                        }
                        n => panic!("Can't index into {:?}", n),
                    }
                }
                current
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

use std::fmt::{Debug, Result as FmtResult, Formatter};

pub struct ConstRustFn(RustFunction);

impl Debug for ConstRustFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "RustFunction")
    }
}
