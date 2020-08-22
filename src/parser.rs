use crate::base::DayObject;
use crate::tokenizer::{DataToken, SymbolToken, Token, KeywordToken};
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

impl Node<'_> {
    pub fn execute(&self, var_mangaer: &mut Variables) -> DayObject {
        match self {
            Node::Data(data) => data.clone(),
            Node::FunctionCall {
                parsed: true,
                id,
                args,
            } => var_mangaer
                .get_var(id)
                .call(args.iter().map(|a| a.execute(var_mangaer)).collect()),
            Node::Identifier(id) => var_mangaer.get_var(id),
            _ => todo!(),
        }
    }
}

pub fn parse<'a, 'b, 'r>(mut tokens: Tokens<'a, 'b, Token<'r>>) -> Vec<Node<'r>> {
    let mut stack: Vec<Node<'r>> = vec![];

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
                                id,
                                parsed: false,
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
                                    content,
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
        }
        println!("Stack: {:?}", stack);
    }
    stack
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
