use regex_lexer::Tokens;
use crate::tokenizer::{Token, DataToken, BracketToken};
use crate::base::DayObject;

enum Node<'a>{
    Data(DayObject),
    FunctionCall{id: &'a str,  args: Vec<Box<dyn ExpressionNode>>},
    Identifier(&'a str),
}

fn parse(tokens: Tokens<Token>) {
    let stack: Vec<Node>;
    for token in tokens {
        match token{
            Token::Data(val) => stack.push(Node::Data(
                match val {
                DataToken::Integer(i) => DayObject::Integer(i),
                DataToken::Float(f) => DayObject::Float(f),
                DataToken::Bool(b) => DayObject::Bool(b),
                DataToken::Character(c) => DayObject::Character(c),
                DataToken::Str(s) => DayObject::Str(s),
            })),
            Token::Identifier(id) => stack.pushNode(Node::Identifier(id)),
            
            Token::Bracket(BracketToken::RoundOpen) => {
                if stack.get(stack.len - 1).is::<Identifier>(){
                    let name = (*stack.pop().unwrap()).downcast_ref::<Identifier>().name;
                    stack.push(Box::new(FunctionCallNode::new(name)));
                }
            }
        }
    }
}