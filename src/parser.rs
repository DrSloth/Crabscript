use crate::base::DayObject;
use crate::tokenizer::{DataToken, SymbolToken, Token, KeywordToken};
use crate::variables::Variables;

use regex_lexer::Tokens;

#[derive(Debug)]
pub enum Node<'a> {
    Data(DayObject),
    FunctionCall {
        id: &'a str,
        args: Option<Vec<Node<'a>>>,
    },
    Identifier(&'a str),
    Assignment {
        id: &'a str,
        value: Box<Option<Node<'a>>>,
    },

    Parentheses {
        parsed: bool,
        content: Vec<Node<'a>>,
    },
}

impl Node<'_> {
    pub fn execute(&self, var_mangaer: &mut Variables) -> DayObject {
        match self {
            Node::Data(data) => data.clone(),
            Node::FunctionCall {
                id,
                args: Some(args),
            } => {
                if let DayObject::Function(func) = var_mangaer.get_var(id) {
                    func.call(args.iter().map(|a| a.execute(var_mangaer)).collect())
                } else {
                    panic!("Err: The function {} does not exist!", id);
                }
            }
            Node::Identifier(id) => var_mangaer.get_var(id),
            Node::Assignment { id, value: v } => {
                /*let node: Node = v.expect("Err: Assignment without value!");
                var_mangaer.def_var(id.to_string(), node.execute(var_mangaer));
                DayObject::None*/
                todo!()
            }
            _ => todo!(),
        }
    }
}

pub fn parse<'a, 'b, 'r>(mut tokens: Tokens<'a, 'b, Token<'r>>) -> Vec<Node<'r>> {
    let mut stack: Vec<Node<'r>> = vec![];
    let mut out: Vec<Node<'r>> = vec![];

    //let mut tokens = tokenizer::TokenStream::new(tokens);

    while let Some(token) = tokens.next() {
        dbg!(&token);
        match token {
            Token::Keyword(k) => {
                let (ts, node) = handle_keyword(k, tokens);
                tokens = ts;
                
            },
            Token::Data(val) => stack.push(handle_data(val)),

            Token::Identifier(id) => stack.push(Node::Identifier(id)),

            Token::Symbol(sym) => match sym {
                SymbolToken::Comma => (),

                SymbolToken::RoundOpen => {
                    let top = stack.pop();
                    match top {
                        Some(Node::Identifier(id)) => {
                            // If an identifier was before the ( it is part of a function call
                            stack.push(Node::FunctionCall {
                                id: id,
                                args: Option::None,
                            });
                        }
                        _ => {
                            panic!("Err: Unexpected Token `(`!");
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
                        let top: Node = stack.pop().expect("Err: Unexpected token `)`!");
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
                                    content,
                                });
                                break;
                            }

                            // Top is the function cll that are colsed by the Token::RoundClosed:
                            Node::FunctionCall {
                                id,
                                args: Option::None,
                            } => {
                                dbg!(&content);
                                stack.push(Node::FunctionCall {
                                    id,
                                    args: Option::Some(content),
                                });
                                dbg!("Err: Function call {} parsed", id);
                                break;
                            }

                            // Top is just content inside the parentheses:
                            _ => content.push(top),
                        }
                    }
                }

                SymbolToken::AssignmentOperator => {
                    let top = stack.pop();
                    if let Some(Node::Identifier(id)) = top {
                        stack.push(Node::Assignment {
                            id,
                            value: Box::new(Option::None),
                        })
                    } else {
                        panic!("Unexpected token `=`!");
                    }
                }

                SymbolToken::DeclarationOperator => todo!(),

                SymbolToken::Semicolon => match stack.len() {
                    0 => panic!("Err: Empty Line!"),
                    1 => out.push(stack.pop().unwrap()),
                    2 => {
                        let none_box: Box<Option<Node>> = Box::from(Option::None); // TODO: Find a better option to match the box
                        match stack.get(0).unwrap() {
                            Node::Assignment {
                                id,
                                value: none_box,
                            } => out.push(Node::Assignment {
                                id,
                                value: Box::new(stack.pop()),
                            }),
                            _ => todo!(),
                        }
                    }
                    _ => panic!("Err: Invalid line! Wrong placement of semicolon?"),
                },
            },
        }
        println!("Stack: {:?}", stack);
    }
    out
}

pub fn handle_data<'a>(data: DataToken) -> Node<'a> {
    Node::Data(match data {
        DataToken::Integer(i) => DayObject::Integer(i),
        DataToken::Float(f) => DayObject::Float(f),
        DataToken::Bool(b) => DayObject::Bool(b),
        DataToken::Character(c) => DayObject::Character(c),
        DataToken::Str(s) => DayObject::Str(s),
    })
}

fn handle_keyword<'a, 'b, 'r>(keyword: KeywordToken, tokens: Tokens<'a, 'b, Token<'r>>) -> (Tokens<'a, 'b, Token<'r>>, Node<'r>) {
    (tokens, match keyword {
        KeywordToken::If => {
            todo!()
        },
        _ => todo!(),
    })
}
