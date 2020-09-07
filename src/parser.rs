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
                SymbolToken::SquareOpen => {
                    let (node, ts) =
                        parse_index(root.pop().expect("can't index into nothing"), tokens);

                    root.push(node);
                    tokens = ts
                }
                SymbolToken::CurlyOpen => {
                    let (node, ts) = parse(tokens);
                    tokens = ts;
                    root.push(Node::RootNode(node))
                }
                SymbolToken::CurlyClose => return (root, tokens),
                t => panic!("unexpected token {:?} {:?}", t, root),
            },
        }
        dbg_print!(&root);
    }
    (root, tokens)
}

pub fn parse_index<'node, 'text, 'tokens>(
    initial: Node<'node>,
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    //TODO implement chained indexing and extract to function
    let mut index_ops = Vec::new();

    loop {
        let (node, ts) = parse_expression(tokens.next().expect("unexpected end of file"), tokens);
        index_ops.push(IndexOperation {
            index: Box::new(node),
        });

        tokens = ts;

        if Some(Token::Symbol(SymbolToken::SquareClose)) != tokens.next() {
            panic!("expected a ]")
        }

        let tok = tokens.next();
        if Some(Token::Symbol(SymbolToken::SquareOpen)) == tok {
            continue;
        }

        if let Some(t) = tok {
            tokens.reinsert(t);
        }
        break;
    }

    let node = Node::Index {
        initial: Box::new(initial),
        index_ops,
    };

    (node, tokens)
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
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    let (node, ts) = match token {
        Token::Data(data) => (parse_data(data), tokens),
        Token::Identifier(id) => parse_ident(id, tokens),
        t => todo!("error handling {:?}", t),
    };

    tokens = ts;

    let next = tokens.next();
    match next {
        Some(Token::Symbol(SymbolToken::SquareOpen)) => {
            parse_index(node, tokens)
        }
        _ => {
            if let Some(t) = next {
                tokens.reinsert(t) 
            }

            (node, tokens)
        } 
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
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
    dbg_print!(&keyword);
    match keyword {
        KeywordToken::Let => parse_declaration(tokens),
        KeywordToken::Const => parse_const_declaration(tokens),
        KeywordToken::If => {
            tokens.reinsert(KeywordToken::If.into());
            let mut branches = vec![];
            loop {
                match tokens.next() {
                    Some(Token::Keyword(KeywordToken::If)) if branches.len() != 0 => {
                        tokens.reinsert(Token::Keyword(KeywordToken::If));
                        break;
                    }
                    next => {
                        let (b, ts) = parse_branch_inner(next, tokens);
                        tokens = ts;
                        if let Some(b) = b {
                            branches.push(b)
                        } else {
                            break;
                        }
                    }
                }
            }

            (Node::BranchNode(branches), tokens)
        }
        KeywordToken::While => {
            let (condition, mut tokens) =
                parse_expression(tokens.next().expect("Unexpected end of file"), tokens);
            dbg_print!(&condition);
            if Some(Token::Symbol(SymbolToken::CurlyOpen)) != tokens.next() {
                panic!("Expected token {")
            }
            let (block, tokens) = parse(tokens);
            dbg_print!(&block);

            (
                Node::While {
                    condition: Box::new(condition),
                    block,
                },
                tokens,
            )
        }
        KeywordToken::Fn => {
            let id: &str;
            if let Some(Token::Identifier(s)) = tokens.next() {
                id = s;
            } else {
                panic!("Fn keyword not followed by identifier");
            }

            if Some(Token::Symbol(SymbolToken::CurlyOpen)) != tokens.next() {
                panic!("Expected token {")
            }
            let (block, tokens) = parse(tokens);
           
            
            (
                Node::FunctionDeclaration{
                    id,
                    block: Some(block),
                },
                tokens
            )
        }
        _ => todo!(),
    }
}

fn parse_branch_inner<'node, 'text, 'tokens>(
    tok: Option<Token<'tokens>>,
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (
    Option<BranchNode<'node>>,
    TokenStream<'node, 'text, 'tokens>,
) {
    if let Some(Token::Keyword(ktok)) = tok {
        match ktok {
            KeywordToken::If => {
                let (condition, block, tokens) = parse_if_inner(tokens);
                (Some(BranchNode::If { condition, block }), tokens)
            }
            KeywordToken::Else => match tokens.next() {
                Some(Token::Keyword(KeywordToken::If)) => {
                    let (condition, block, tokens) = parse_if_inner(tokens);
                    (Some(BranchNode::ElseIf { condition, block }), tokens)
                }
                Some(Token::Symbol(SymbolToken::CurlyOpen)) => {
                    let (block, tokens) = parse(tokens);
                    (Some(BranchNode::Else { block }), tokens)
                }
                None => panic!("Unexpected end of file"),
                t => panic!("Unexpectedt token {:?}", t),
            },
            _ => {
                tokens.reinsert(ktok.into());
                (None, tokens)
            }
        }
    } else {
        if let Some(tok) = tok {
            tokens.reinsert(tok.into())
        }

        (None, tokens)
    }
}

fn parse_if_inner<'node, 'text, 'tokens>(
    mut tokens: TokenStream<'node, 'text, 'tokens>,
) -> (
    Box<Node<'node>>,
    RootNode<'node>,
    TokenStream<'node, 'text, 'tokens>,
) {
    let (condition, mut tokens) =
        parse_expression(tokens.next().expect("Unexpected end of file"), tokens);
    if Some(Token::Symbol(SymbolToken::CurlyOpen)) != tokens.next() {
        panic!("expected a {")
    }

    let (block, tokens) = parse(tokens);
    (Box::new(condition), block, tokens)
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

    (id, Box::new(node), ts)
}
