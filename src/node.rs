use crate::std_modules::conversion;
use crate::{base::DayObject, variables::Variables};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Node<'a> {
    RootNode(RootNode<'a>),
    Data(DayObject),
    FunctionCall {
        id: &'a str,
        args: Vec<Node<'a>>,
    },
    Identifier(&'a str),
    Declaration {
        id: &'a str,
        value: Box<Node<'a>>,
    },
    Assignment {
        id: &'a str,
        value: Box<Node<'a>>,
    },
    ConstDeclaration {
        id: &'a str,
        value: Box<Node<'a>>,
    },
    FunctionDeclaration {
        id: &'a str,
        block: Option<RootNode<'a>>,
    },
    Parentheses {
        parsed: bool,
        content: Vec<Node<'a>>,
    },
    While {
        condition: Box<Node<'a>>,
        block: RootNode<'a>,
    },
    BranchNode(Vec<BranchNode<'a>>),
    Index {
        initial: Box<Node<'a>>,
        index_ops: Vec<IndexOperation<'a>>,
    },
    Ret(Option<Arc<Node<'a>>>),
}

impl<'a: 'v, 'v, 's> Node<'a> {
    pub fn execute(&self, var_manager: Arc<Variables<'v>>) -> DayObject {
        match self {
            Node::Data(data) => data.clone(),
            Node::FunctionCall { id, args } => {
                if let DayObject::Function(func) = var_manager.clone().get_var(id) {
                    let mut ar = vec![];
                    for a in args {
                        ar.push(a.execute(Arc::clone(&var_manager)))
                    }

                    func.call(ar, Arc::clone(&var_manager))
                } else {
                    panic!("Err: The function {} does not exist!", id);
                }
            }
            Node::Identifier(id) => var_manager.get_var(id),
            Node::Declaration { id, value: v } => {
                let value = v.execute(Arc::clone(&var_manager));
                var_manager.def_var(id.to_string(), value);
                DayObject::None
            }
            Node::ConstDeclaration { id, value: v } => {
                let value = v.execute(Arc::clone(&var_manager));
                var_manager.def_const(id.to_string(), value);
                DayObject::None
            }
            Node::Assignment { id, value: v } => {
                let value = v.execute(Arc::clone(&var_manager));
                var_manager.set_var(id, value);
                DayObject::None
            }
            Node::BranchNode(branches) => {
                for b in branches {
                    match b {
                        BranchNode::If { condition, block } => {
                            if let DayObject::Bool(true) =
                                condition.execute(Arc::clone(&var_manager))
                            {
                                block.execute(Arc::clone(&var_manager));
                                break;
                            }
                        }
                        BranchNode::ElseIf { condition, block } => {
                            if let DayObject::Bool(true) =
                                condition.execute(Arc::clone(&var_manager))
                            {
                                block.execute(Arc::clone(&var_manager));
                                break;
                            }
                        }
                        BranchNode::Else { block } => {
                            block.execute(Arc::clone(&var_manager));
                            break;
                        }
                    }
                }

                DayObject::None
            }
            Node::While { condition, block } => {
                while let DayObject::Bool(true) = condition.execute(Arc::clone(&var_manager)) {
                    block.execute(Arc::clone(&var_manager));
                }

                DayObject::None
            }
            Node::FunctionDeclaration { id, block } => {
                //NOTE Move function declaration out of node, populate var_manager in parser
                //The old system didn't suit closures or inner functions this system costs too much
                if let Some(b) = block.clone() {
                    var_manager.def_fn(id.to_string(), b);
                }
                DayObject::None
            }
            Node::Index { initial, index_ops } => {
                let mut current = initial.execute(Arc::clone(&var_manager));

                for i in index_ops {
                    current = match current {
                        DayObject::Array(a) => {
                            a[conversion::to_int_inner(i.index.execute(Arc::clone(&var_manager)))
                                as usize]
                                .clone()
                        }
                        n => panic!("Can't index into {:?}", n),
                    }
                }

                current
            }

            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BranchNode<'a> {
    If {
        condition: Box<Node<'a>>,
        block: RootNode<'a>,
    },
    ElseIf {
        condition: Box<Node<'a>>,
        block: RootNode<'a>,
    },
    Else {
        block: RootNode<'a>,
    },
}

#[derive(Debug, Clone)]
pub struct RootNode<'a>(Vec<Node<'a>>, NodePurpose);

impl<'a: 'v, 'v, 's> RootNode<'a> {
    pub fn new(purpose: NodePurpose) -> Self {
        Self(vec![], purpose)
    }

    pub fn push(&mut self, node: Node<'a>) {
        self.0.push(node)
    }

    pub fn pop(&mut self) -> Option<Node<'a>> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn execute(&'s self, var_manager: Arc<Variables<'v>>) -> Return {
        for n in self.0.iter() {
            match n {
                Node::Ret(expr) => {
                    return if self.1 == NodePurpose::Function {
                        if let Some(expr) = expr {
                            Return::Value(expr.execute(var_manager))
                        } else {
                            Return::Value(DayObject::None)
                        }
                    } else {
                        if let Some(expr) = expr {
                            Return::Return(Arc::clone(expr))
                        } else {
                            Return::Value(DayObject::None)
                        }
                    }
                }
                n => n.execute(Arc::clone(&var_manager)),
            };
        }

        Return::Finished
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NodePurpose {
    Function,
    Conditional,
    While,
    Block,
    TopLevel,
}

impl<'a> IntoIterator for RootNode<'a> {
    type Item = Node<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct IndexOperation<'a> {
    pub index: Box<Node<'a>>,
}

pub enum Return<'a> {
    Finished,
    Return(Arc<Node<'a>>),
    Value(DayObject),
    Yielded(DayObject),
}
