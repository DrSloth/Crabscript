use crate::{base::DayObject, variables::Variables};

#[derive(Debug)]
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
    ConstDeclaration
    {
        id: &'a str,
        value: Box<Node<'a>>,
    },
    Parentheses {
        parsed: bool,
        content: Vec<Node<'a>>,
    },
    If {
        condition: Box<Node<'a>>,
        block: RootNode<'a>,
    }
}

impl Node<'_> {
    pub fn execute(&self, var_mangaer: &mut Variables) -> DayObject {
        match self {
            Node::Data(data) => data.clone(),
            Node::FunctionCall { id, args } => {
                if let DayObject::Function(func) = var_mangaer.get_var(id) {
                    func.call(args.iter().map(|a| a.execute(var_mangaer)).collect())
                } else {
                    panic!("Err: The function {} does not exist!", id);
                }
            }
            Node::Identifier(id) => var_mangaer.get_var(id),
            Node::Declaration { id, value: v } => {
                let value = v.execute(var_mangaer);
                var_mangaer.def_var(id.to_string(), value);
                DayObject::None
            }
            Node::ConstDeclaration { id, value: v } => {
                let value = v.execute(var_mangaer);
                var_mangaer.def_const(id.to_string(), value);
                DayObject::None
            }
            Node::Assignment { id, value: v } => {
                let value = v.execute(var_mangaer);
                var_mangaer.set_var(id, value);
                DayObject::None
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
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
        for n in self.0.iter()  {
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
