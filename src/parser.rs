use crate::base::DayObject;
use crate::io::print;
use crate::tokenizer::{DataToken, SymbolToken, Token};
use crate::variables::Variables;

use regex_lexer::Tokens;

#[derive(Debug)]
pub enum Node<'a> {
    Data(DayObject),
    FunctionCall {
        parsed: bool,
        id: &'a str,
        args: Vec<Node<'a>>,
    },
    Identifier(&'a str),

    Parentheses {
        parsed: bool,
        content: Vec<Node<'a>>,
    },
}

enum Expression {}

impl Node<'_> {
    pub fn execute(&self) -> DayObject {
        match self {
            Node::Data(data) => data.clone(),
            Node::FunctionCall {
                parsed: true,
                id: id,
                args: args,
            } => call_function(id, args.iter().map(|a| a.execute()).collect()),
            _ => todo!(),
        }
    }
}

// TEMP: Has to be implemented properly in a seperate crate
fn call_function(id: &str, args: Vec<DayObject>) -> DayObject {
    println!("{:?}",Variables::new() );
    if let DayObject::Function(func) = Variables::new().get_var(id)  {
        func(args)
    } else {
        panic!("The function {} does not exist!", id);
    }
}

pub fn parse<'a>(tokens: Tokens<Token<'a>>) -> Vec<Node<'a>> {
    let mut stack: Vec<Node<'a>> = vec![];
    for token in tokens {
        dbg!(&token);
        match token {
            Token::Data(val) => stack.push(Node::Data(match val {
                DataToken::Integer(i) => DayObject::Integer(i),
                DataToken::Float(f) => DayObject::Float(f),
                DataToken::Bool(b) => DayObject::Bool(b),
                DataToken::Character(c) => DayObject::Character(c),
                DataToken::Str(s) => DayObject::Str(s),
            })),

            Token::Identifier(id) => stack.push(Node::Identifier(id)),

            Token::Symbol(sym) => match sym {
                SymbolToken::RoundOpen => {
                    let top = stack.pop();
                    match top {
                        Some(Node::Identifier(id)) => {
                            // If an identifier was before the ( it is part of a function call
                            stack.push(Node::FunctionCall {
                                parsed: false,
                                id: id,
                                args: vec![],
                            });
                        }
                        _ => {
                            panic!("Unexpected Token `(`!");
                            /*
                            stack.push(top);
                            stack.push(Node::Parentheses {
                                // Else its just part of regural parentheses
                                parsed: false,
                                content: vec![],
                            });
                            */
                        }
                    }
                }

                SymbolToken::RoundClose => {
                    let mut content: Vec<Node> = Vec::new();
                    loop {
                        let top: Node = stack.pop().expect("Unexpected token `)`!");
                        dbg!(&top);

                        match top {
                            // Top is the parentheses that are colsed by the Token::RoundClosed:
                            Node::Parentheses {
                                parsed: false,
                                content: mut previous_content,
                            } => {
                                content.append(&mut previous_content);
                                stack.push(Node::Parentheses {
                                    parsed: true,
                                    content: content,
                                });
                                break;
                            }

                            // Top is the function cll that are colsed by the Token::RoundClosed:
                            Node::FunctionCall {
                                parsed: false,
                                id,
                                args: mut previous_conent,
                            } => {
                                dbg!(&content);
                                content.append(&mut previous_conent);
                                stack.push(Node::FunctionCall {
                                    parsed: true,
                                    id,
                                    args: content,
                                });
                                dbg!("Function call {} parsed", id);
                                break;
                            }

                            // Top is just content inside the parentheses:
                            _ => content.push(top),
                        }
                    }
                }
                SymbolToken::AssignmentOperator => todo!(),
                SymbolToken::DeclarationOperator => todo!(),
            },

            _ => todo!(),
        }
        println!("Stack: {:?}", stack);
    }
    stack
}
