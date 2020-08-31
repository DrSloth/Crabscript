use crate::std_modules::conversion;
use crate::{base::{DayObject, DayFunction}, variables::Variables};

#[derive(Debug, Clone)]
pub enum Node<'a> {
    RootNode(RootNode<'a>),
    Data(DayObject<'a>),
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
        block: RootNode<'a>,
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
        //NOTE this is recursive and thus could be optimised
        //it would be better if this would go from left to right instead
        //of right to left
        initial: Box<Node<'a>>,
        index_ops: Vec<IndexOperation<'a>>,
    },
}

impl Node<'_> {
    pub fn execute(&mut self, var_manager: &mut Variables) -> DayObject {
        match self {
            Node::Data(data) => data.clone(),
            Node::FunctionCall { id, args } => {
                if let DayObject::Function(func) = var_manager.get_var(id) {
                    func.call(args.iter_mut().map(|a| a.execute(var_manager)).collect())
                } else {
                    panic!("Err: The function {} does not exist!", id);
                }
            }
            Node::Identifier(id) => var_manager.get_var(id),
            Node::Declaration { id, value: v } => {
                let value = v.execute(var_manager);
                var_manager.def_var(id.to_string(), value);
                DayObject::None
            }
            Node::ConstDeclaration { id, value: v } => {
                let value = v.execute(var_manager);
                var_manager.def_const(id.to_string(), value);
                DayObject::None
            }
            Node::Assignment { id, value: v } => {
                let value = v.execute(var_manager);
                var_manager.set_var(id, value);
                DayObject::None
            }
            Node::BranchNode(branches) => {
                for b in branches {
                    match b {
                        BranchNode::If { condition, block } => {
                            if let DayObject::Bool(true) = condition.execute(var_manager) {
                                block.execute(var_manager);
                                break;
                            }
                        }
                        BranchNode::ElseIf { condition, block } => {
                            if let DayObject::Bool(true) = condition.execute(var_manager) {
                                block.execute(var_manager);
                                break;
                            }
                        }
                        BranchNode::Else { block } => {
                            block.execute(var_manager);
                            break;
                        }
                        }
                }

                DayObject::None
            }
            Node::While { condition, block } => {
                while let DayObject::Bool(true) = condition.execute(var_manager) {
                    block.execute(var_manager)
                }

                DayObject::None
            }
            Node::FunctionDeclaration {id, block} => {
                var_manager.def_var(id.to_string(), DayObject::Function(DayFunction::RuntimeDef(block, var_manager)));
                DayObject::None
            }
            Node::Index { initial, index_ops } => {
                let mut current = initial.execute(var_manager);

                for i in index_ops {
                    current = match current {
                        DayObject::Array(a) => a
                            [conversion::to_int_inner(i.index.execute(var_manager)) as usize]
                            .clone(),
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
pub struct RootNode<'a>(Vec<Node<'a>>);

impl<'a> RootNode<'a> {
    pub fn new() -> Self {
        Self(vec![])
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

    pub fn execute(&mut self, var_manager: &mut Variables) {
        for n in self.0.iter_mut() {
            n.execute(var_manager);
        }
    }
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
