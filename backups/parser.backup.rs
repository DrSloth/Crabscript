use crate::{
    base::DayObject,
    node::*,
    tokenizer::{DataToken, KeywordToken, SymbolToken, Token, TokenStream},
};

pub fn parse<'node, 'text, 'tokens>(
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (RootNode<'node>, TokenStream<'node, 'text, 'tokens>) {
    let mut root = RootNode::new();

    while let Some(token) = tokens.next() {
        dbg_print!(&token);
        match token {
            Token::Keyword(k) => {
                let (node, ts) = parse_keyword(k, tokens);
                tokens = ts;
                root.push(node)
            }
            Token::Data(val) => root.push(parse_data(val)),

            Token::Identifier(id) => {
                let (node, ts) = parse_ident(id, tokens);
                tokens = ts;
                root.push(node)
            }

            Token::Symbol(sym) => match sym {
                SymbolToken::CurlyOpen => {
                    let (node, ts) = parse(tokens);
                    tokens = ts;
                    root.push(Node::RootNode(node))
                }
                SymbolToken::CurlyClose => return (root, tokens),
                SymbolToken::Comma => (),

                SymbolToken::RoundOpen => {
                    let top = root.pop();
                    match top {
                        /*Some(Node::Identifier(id)) => {
                            // If an identifier was before the ( it is part of a function call
                            root.push(Node::FunctionCall {
                                id: id,
                                args: Option::None,
                            });
                        }*/
                        _ => {
                            panic!("Err: Unexpected Token `(`!");
                            /*
                            root.push(top);
                            root.push(Node::Parentheses {
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
                        let top: Node = root.pop().expect("Err: Unexpected token `)`!");
                        dbg_print!(&top);

                        match top {
                            // Top is the parentheses that are colsed by the Token::RoundClosed:
                            Node::Parentheses {
                                parsed: false,
                                content: mut previous_content,
                            } => {
                                content.append(&mut previous_content);
                                root.push(Node::Parentheses {
                                    parsed: true,
                                    content,
                                });
                                break;
                            }

                            // Top is the function cll that are colsed by the Token::RoundClosed:
                            /* Node::FunctionCall {
                                id,
                                args: Option::None,
                            } => {
                                dbg_print!(&content);
                                root.push(Node::FunctionCall {
                                    id,
                                    args: Some(content),
                                });
                                dbg_print!(format!("Err: Function call {} parsed", id));
                                break;
                            }*/
                            // Top is just content inside the parentheses:
                            _ => content.push(top),
                        }
                    }
                }

                /*SymbolToken::AssignmentOperator => {
                    let top = root.pop();
                    if let Some(Node::Identifier(id)) = top {
                        root.push(Node::Assignment {
                            id,
                            value: Box::new(Option::None),
                        })
                    } else {
                        panic!("Unexpected token `=`!");
                    }
                }

                SymbolToken::DeclarationOperator => todo!(),*/
                SymbolToken::Semicolon => match root.len() {
                    /*0 => panic!("Err: Empty Line!"),
                    1 => out.push(root.pop().unwrap()),
                    2 => {
                        let none_box: Box<Option<Node>> = Box::from(Option::None); // TODO: Find a better option to match the box
                        match root.get(0).unwrap() {
                            Node::Assignment {
                                id,
                                value: none_box,
                            } => out.push(Node::Assignment {
                                id,
                                value: Box::new(root.pop()),
                            }),
                            _ => todo!(),
                        }
                    }
                    _ => panic!("Err: Invalid line! Wrong placement of semicolon?"),*/
                    _ => todo!(),
                },
                t => panic!("unexpected token {:?} {:?}", t, root),
            },
        }
        dbg_print!(&root);
    }
    (root, tokens)
}

///parses anything starting with an ident
pub fn parse_ident<'node, 'text, 'tokens>(
    identifier: &'node str,
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    let next = tokens.next();
    match next {
        None => (Node::Identifier(identifier), tokens),
        Some(Token::Symbol(SymbolToken::RoundOpen)) => parse_function(identifier, tokens),
        Some(Token::Symbol(SymbolToken::Equals)) => {
            let (node, ts) = parse_expression(
                tokens.next().expect("unexpected end of file after ="),
                tokens,
            );

            (
                Node::Assignment {
                    id: identifier,
                    value: Box::new(node),
                },
                ts,
            )
        }
        Some(token) => {
            tokens.reinsert(token);
            (Node::Identifier(identifier), tokens)
        }
    }
}

pub fn parse_function<'node, 'text, 'tokens>(
    identifier: &'node str,
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    let mut args: Vec<Node<'node>> = vec![];
    //NOTE At the moment the parsing of functions is partially recursive (a function in a function is recursive) but that
    //shouldn't be too hard to fix

    loop {
        let (arg, ts) = parse_arg(tokens.next().expect("unexpected end of file"), tokens);
        tokens = ts;
        dbg_print!(&arg);
        if let Some(n) = arg {
            let next = tokens.next();
            match next {
                None => panic!("Unexpected end of file"),
                Some(Token::Symbol(SymbolToken::Comma)) => args.push(n),
                Some(Token::Symbol(SymbolToken::RoundClose)) => {
                    args.push(n);
                    break;
                }
                Some(t) => panic!("unexpected token {:?}", t),
            }
        } else {
            break;
        }
    }

    (
        Node::FunctionCall {
            id: identifier,
            args,
        },
        tokens,
    )
}

pub fn parse_arg<'node, 'text, 'tokens>(
    token: Token<'tokens>,
    tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Option<Node<'node>>, TokenStream<'node, 'text, 'tokens>) {
    match token {
        Token::Symbol(SymbolToken::RoundClose) => (None, tokens),
        t => {
            let (node, ts) = parse_expression(t, tokens);
            (Some(node), ts)
        }
    }
}

pub fn parse_expression<'node, 'text, 'tokens>(
    token: Token<'tokens>,
    tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    match token {
        Token::Data(data) => (parse_data(data), tokens),
        Token::Identifier(id) => parse_ident(id, tokens),
        t => todo!("error handling hehe {:?}", t),
    }
}

pub fn parse_data<'node>(data: DataToken) -> Node<'node> {
    Node::Data(match data {
        DataToken::Integer(i) => DayObject::Integer(i),
        DataToken::Float(f) => DayObject::Float(f),
        DataToken::Bool(b) => DayObject::Bool(b),
        DataToken::Character(c) => DayObject::Character(c),
        DataToken::Str(s) => DayObject::Str(s),
    })
}

fn parse_keyword<'node, 'text, 'tokens>(
    keyword: KeywordToken,
    tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    dbg_print!(&keyword);
    match keyword {
        KeywordToken::Let => parse_declaration(tokens),
        KeywordToken::Const => parse_const_declaration(tokens),
        KeywordToken::If => {
            //The nect thing to do is extract identifier and function call parsing
            //after that this can be done
            todo!()
        }
        _ => todo!(),
    }
}

fn parse_declaration<'node, 'text, 'tokens>(
    tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    let decl = decl_inner(tokens);
    (
        Node::Declaration {
            id: decl.0,
            value: decl.1,
        },
        decl.2,
    )
}

fn parse_const_declaration<'node, 'text, 'tokens>(
    tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    let decl = decl_inner(tokens);
    (
        Node::ConstDeclaration {
            id: decl.0,
            value: decl.1,
        },
        decl.2,
    )
}

fn decl_inner<'node, 'text, 'tokens>(
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (
    &'node str,
    Box<Node<'node>>,
    TokenStream<'node, 'text, 'tokens>,
) {
    //NOTE currently testing this little cool macro
    let id = expect!(tokens.next().expect("Unexpected end of file") => Token::Identifier | "Expected identifier");
    if Some(Token::Symbol(SymbolToken::Equals)) != tokens.next() {
        panic!("A declaration needs an equals");
    }
    let (node, ts) = parse_expression(
        tokens.next().expect("unexpected end of file after ="),
        tokens,
    );

    (
        
        id,
        Box::new(node),
        ts,
    )
}
