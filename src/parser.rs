use crate::base::DayObject;
use crate::tokenizer::{BracketToken, DataToken, Token};
use regex_lexer::Tokens;

enum Node<'a> {
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

fn parse(tokens: Tokens<Token>) {
    let mut stack: Vec<Node> = vec![];
    for token in tokens {
        match token {
            Token::Data(val) => stack.push(Node::Data(match val {
                DataToken::Integer(i) => DayObject::Integer(i),
                DataToken::Float(f) => DayObject::Float(f),
                DataToken::Bool(b) => DayObject::Bool(b),
                DataToken::Character(c) => DayObject::Character(c),
                DataToken::Str(s) => DayObject::Str(s),
            })),

            Token::Identifier(id) => stack.push(Node::Identifier(id)),

            Token::Bracket(BracketToken::RoundOpen) => match stack.get(stack.len() - 1) {
                Some(Node::Identifier(id)) => {
                    // If an identifier was before the ( it is part of a function call
                    stack.push(Node::FunctionCall {
                        parsed: false,
                        id: id,
                        args: vec![],
                    });
                    stack.pop();
                }
                _ => stack.push(Node::Parentheses {
                    // Else its just part of regural parentheses
                    parsed: false,
                    content: vec![],
                }),
            },
            Token::Bracket(BracketToken::RoundClose) => loop {
                let mut content: Vec<Node> = Vec::new();
                let top: Node = stack.pop().expect("`)` umatched!");

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
                        content.append(&mut previous_conent);
                        stack.push(Node::FunctionCall {
                            parsed: true,
                            id,
                            args: content,
                        });
                        break;
                    }

                    // Top is just content inside the parentheses:
                    _ => content.push(top),
                }
            },

            _ => todo!(),
        }
    }
}
