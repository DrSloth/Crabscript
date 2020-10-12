use crate::{
    base::DayObject,
    node::*,
    tokenizer::{DataToken, KeywordToken, SymbolToken, Token, TokenStream},
};
use std::sync::Arc;

pub struct Parser {
    curr_line: u64,
}

impl Parser {
    pub fn new() -> Self {
        Parser { curr_line: 1 }
    }

    pub fn parse<'node, 'text, 'tokens>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        purpose: NodePurpose,
    ) -> (RootNode<'node>, TokenStream<'node, 'text, 'tokens>) {
        let mut root = RootNode::new(purpose);

        while let Some(token) = self.next_token(&mut tokens) {
            dbg_print!(&token);
            match token {
                Token::Keyword(k) => {
                    let (node, ts) = self.parse_keyword(k, tokens);
                    tokens = ts;
                    root.push(node)
                }
                Token::Data(val) => root.push(self.parse_data(val)),

                Token::Identifier(id) => {
                    let (node, ts) = self.parse_ident(id, tokens);
                    tokens = ts;
                    root.push(node)
                }

                Token::Symbol(sym) => match sym {
                    SymbolToken::SquareOpen => {
                        let (node, ts) =
                            self.parse_index(root.pop().expect("can't index into nothing"), tokens);

                        root.push(node);
                        tokens = ts
                    }
                    SymbolToken::CurlyOpen => {
                        let (node, ts) = self.parse(tokens, NodePurpose::Block);
                        tokens = ts;
                        root.push(Node::RootNode(node))
                    }
                    SymbolToken::CurlyClose => return (root, tokens),
                    t => panic!("unexpected token {:?} {:?}", t, root),
                },
                Token::Newline => {
                    dbg_print!(self.curr_line);
                    self.curr_line += 1
                }
            }
            dbg_print!(&root);
        }
        println!(" >> Parsed {} lines", self.curr_line);
        (root, tokens)
    }

    pub fn parse_index<'node, 'text, 'tokens>(
        &mut self,
        initial: Node<'node>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
        //TODO implement chained indexing and extract to function
        let mut index_ops = Vec::new();

        loop {
            let next_token = self
                .next_token(&mut tokens)
                .expect("Unexpected end of file");

            let (node, ts) = self.parse_expression(next_token, tokens);
            index_ops.push(IndexOperation {
                index: Box::new(node),
            });

            tokens = ts;

            if Some(Token::Symbol(SymbolToken::SquareClose)) != self.next_token(&mut tokens) {
                panic!("expected a ]")
            }

            let tok = self.next_token(&mut tokens);
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
        &mut self,
        identifier: &'node str,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
        let next = self.next_token(&mut tokens);
        match next {
            None => (Node::Identifier(identifier), tokens),
            Some(Token::Symbol(SymbolToken::RoundOpen)) => self.parse_function(identifier, tokens),
            Some(Token::Symbol(SymbolToken::Equals)) => {
                let next_token = self
                    .next_token(&mut tokens)
                    .expect("Unexpected end of file");

                let (node, ts) = self.parse_expression(next_token, tokens);

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
        &mut self,
        identifier: &'node str,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
        let mut args: Vec<Node<'node>> = vec![];
        //NOTE At the moment the parsing of functions is partially recursive (a function in a function is recursive) but that
        //shouldn't be too hard to fix

        loop {
            let next_token = self
                .next_token(&mut tokens)
                .expect("Unexpected end of file");

            let (arg, ts) = self.parse_arg(next_token, tokens);
            tokens = ts;
            dbg_print!(&arg);
            if let Some(n) = arg {
                let next = self.next_token(&mut tokens);
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
        &mut self,
        token: Token<'tokens>,
        tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Option<Node<'node>>, TokenStream<'node, 'text, 'tokens>) {
        match token {
            Token::Symbol(SymbolToken::RoundClose) => (None, tokens),
            t => {
                let (node, ts) = self.parse_expression(t, tokens);
                (Some(node), ts)
            }
        }
    }

    pub fn parse_expression<'node, 'text, 'tokens>(
        &mut self,
        token: Token<'tokens>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
        let (node, ts) = match token {
            Token::Data(data) => (self.parse_data(data), tokens),
            Token::Identifier(id) => self.parse_ident(id, tokens),
            Token::Keyword(key) => self.parse_keyword(key, tokens),
            t => todo!("error handling {:?}", t),
        };

        tokens = ts;

        let next = self.next_token(&mut tokens);
        match next {
            Some(Token::Symbol(SymbolToken::SquareOpen)) => self.parse_index(node, tokens),
            _ => {
                if let Some(t) = next {
                    tokens.reinsert(t)
                }

                (node, tokens)
            }
        }
    }

    pub fn parse_data<'node>(&mut self, data: DataToken) -> Node<'node> {
        Node::Data(match data {
            DataToken::Integer(i) => DayObject::Integer(i),
            DataToken::Float(f) => DayObject::Float(f),
            DataToken::Bool(b) => DayObject::Bool(b),
            DataToken::Character(c) => DayObject::Character(c),
            DataToken::Str(s) => DayObject::Str(s),
        })
    }

    fn parse_keyword<'node, 'text, 'tokens>(
        &mut self,
        keyword: KeywordToken,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
        dbg_print!(&keyword);
        match keyword {
            KeywordToken::Ret => {
                let (expr, ts) = self.parse_ret(tokens);
                match expr {
                    Some(expr) => (Node::Ret(Some(Arc::new(expr))), ts),
                    None => (Node::Ret(None), ts),
                }
            }
            KeywordToken::Let => self.parse_declaration(tokens),
            KeywordToken::Const => self.parse_const_declaration(tokens),
            KeywordToken::If => {
                tokens.reinsert(KeywordToken::If.into());
                let mut branches = vec![];
                loop {
                    match self.next_token(&mut tokens) {
                        Some(Token::Keyword(KeywordToken::If)) if branches.len() != 0 => {
                            tokens.reinsert(Token::Keyword(KeywordToken::If));
                            break;
                        }
                        next => {
                            let (b, ts) = self.parse_branch_inner(next, tokens);
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
                let next_token = self
                    .next_token(&mut tokens)
                    .expect("Unexpected end of file");
                let (condition, mut tokens) = self.parse_expression(next_token, tokens);
                dbg_print!(&condition);
                if Some(Token::Symbol(SymbolToken::CurlyOpen)) != self.next_token(&mut tokens) {
                    panic!("Expected token {")
                }
                let (block, tokens) = self.parse(tokens, NodePurpose::While);
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
                let next = self.next_token(&mut tokens);
                let id = if let Some(Token::Identifier(s)) = next {
                    Some(s)
                } else {
                    if let Some(next) = next {
                        tokens.reinsert(next);
                    } else {
                        panic!("Expected token {")
                    }
                    None
                };

                if Some(Token::Symbol(SymbolToken::CurlyOpen)) != self.next_token(&mut tokens) {
                    panic!("Expected token {")
                }
                let (block, tokens) = self.parse(tokens, NodePurpose::Function);

                (Node::function_decl(id, block), tokens)
            }
            _ => todo!(),
        }
    }

    fn parse_ret<'node, 'text, 'tokens>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Option<Node<'node>>, TokenStream<'node, 'text, 'tokens>) {
        match self.next_token(&mut tokens) {
            None => (None, tokens),
            Some(t) => {
                let expr = self.parse_expression(t, tokens);
                (Some(expr.0), expr.1)
            }
        }
    }

    fn parse_branch_inner<'node, 'text, 'tokens>(
        &mut self,
        tok: Option<Token<'tokens>>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (
        Option<BranchNode<'node>>,
        TokenStream<'node, 'text, 'tokens>,
    ) {
        if let Some(Token::Keyword(ktok)) = tok {
            match ktok {
                KeywordToken::If => {
                    let (condition, block, tokens) = self.parse_if_inner(tokens);
                    (Some(BranchNode::If { condition, block }), tokens)
                }
                KeywordToken::Else => match self.next_token(&mut tokens) {
                    Some(Token::Keyword(KeywordToken::If)) => {
                        let (condition, block, tokens) = self.parse_if_inner(tokens);
                        (Some(BranchNode::ElseIf { condition, block }), tokens)
                    }
                    Some(Token::Symbol(SymbolToken::CurlyOpen)) => {
                        let (block, tokens) = self.parse(tokens, NodePurpose::Conditional);
                        (Some(BranchNode::Else { block }), tokens)
                    }
                    None => panic!("Unexpected end of file"),
                    t => panic!("Unexpected token {:?}", t),
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
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (
        Box<Node<'node>>,
        RootNode<'node>,
        TokenStream<'node, 'text, 'tokens>,
    ) {
        let next_token = self
            .next_token(&mut tokens)
            .expect("Unexpected end of file");

        let (condition, mut tokens) = self.parse_expression(next_token, tokens);
        if Some(Token::Symbol(SymbolToken::CurlyOpen)) != self.next_token(&mut tokens) {
            panic!("expected a {")
        }

        let (block, tokens) = self.parse(tokens, NodePurpose::Conditional);
        (Box::new(condition), block, tokens)
    }

    fn parse_declaration<'node, 'text, 'tokens>(
        &mut self,
        tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
        let decl = self.decl_inner(tokens);
        (
            Node::Declaration {
                id: decl.0,
                value: decl.1,
            },
            decl.2,
        )
    }

    fn parse_const_declaration<'node, 'text, 'tokens>(
        &mut self,
        tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (Node<'node>, TokenStream<'node, 'text, 'tokens>) {
        let decl = self.decl_inner(tokens);
        (
            Node::ConstDeclaration {
                id: decl.0,
                value: decl.1,
            },
            decl.2,
        )
    }

    fn decl_inner<'node, 'text, 'tokens>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> (
        &'node str,
        Box<Node<'node>>,
        TokenStream<'node, 'text, 'tokens>,
    ) {
        //NOTE currently testing this little cool macro
        let id = expect!(self.next_token(&mut tokens).expect("Unexpected end of file") => Token::Identifier | "Expected identifier");
        if Some(Token::Symbol(SymbolToken::Equals)) != self.next_token(&mut tokens) {
            panic!("A declaration needs an equals");
        }
        let next_token = self
            .next_token(&mut tokens)
            .expect("unexpected end of file after =");
        let (node, ts) = self.parse_expression(next_token, tokens);

        (id, Box::new(node), ts)
    }

    /// Retruns the next non-meta token
    /// And handles the meta-tokens,
    /// by e.g. incrementing line numbers.
    fn next_token<'node, 'text, 'tokens>(
        &mut self,
        tokens: &mut TokenStream<'node, 'text, 'tokens>,
    ) -> Option<Token<'tokens>> {
        loop {
            match tokens.next() {
                Some(Token::Newline) => {
                    self.curr_line += 1;
                }
                other_token => {
                    return other_token;
                }
            }
        }
    }
}
