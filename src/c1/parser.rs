use super::parsing_error::{ParsingError, ParsingErrorKind, ParsingResult};
use crate::{
    base::DayObject,
    hash,
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
    ) -> ParsingResult<(RootNode<'node>, TokenStream<'node, 'text, 'tokens>)> {
        let mut root = RootNode::new(purpose);

        while let Ok(token) = self.next_token(&mut tokens) {
            dbg_print!(&token);
            match token {
                Token::Keyword(k) => {
                    let (node, ts) = self.parse_keyword(k, tokens)?;
                    tokens = ts;
                    root.push(node)
                }
                Token::Data(val) => root.push(self.parse_data(val)),

                Token::Identifier(id) => {
                    let (node, ts) = self.parse_ident(id, tokens)?;
                    tokens = ts;
                    root.push(node)
                }

                Token::Symbol(sym) => match sym {
                    SymbolToken::SquareOpen => {
                        let (node, ts) = self.parse_index(
                            match root.pop() {
                                Some(n) => n,
                                None => {
                                    return Err(ParsingError::new(
                                        ParsingErrorKind::UnexpectedEndOfInput,
                                        self.curr_line,
                                    ))
                                }
                            },
                            tokens,
                        )?;

                        root.push(node);
                        tokens = ts
                    }
                    SymbolToken::CurlyOpen => {
                        let (node, ts) = self.parse(tokens, NodePurpose::Block)?;
                        tokens = ts;
                        root.push(Node::RootNode(node))
                    }
                    SymbolToken::CurlyClose => {
                        if let NodePurpose::TopLevel = root.purpose {
                            return Err(ParsingError::new(
                                ParsingErrorKind::Unexpected("}".to_string()),
                                self.curr_line,
                            ));
                        } else {
                            return Ok((root, tokens));
                        }
                    }
                    SymbolToken::RoundOpen => {
                        let expr = root.pop().ok_or(ParsingError::new(
                            ParsingErrorKind::ExpectedNotFound("Preceeding function".to_string()),
                            self.curr_line,
                        ))?;
                        let (node, ts) = self.parse_call(expr, tokens)?;
                        tokens = ts;
                        root.push(node);
                    }
                    t => {
                        return Err(ParsingError::new(
                            ParsingErrorKind::Unexpected(format!("{:?}", t)),
                            self.curr_line,
                        ))
                    }
                },
                Token::Newline => {
                    dbg_print!(self.curr_line);
                    self.curr_line += 1
                }
            }
            dbg_print!(&root);
        }
        Ok((root, tokens))
    }

    pub fn parse_index<'node, 'text, 'tokens>(
        &mut self,
        initial: Node<'node>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> Result<(Node<'node>, TokenStream<'node, 'text, 'tokens>), ParsingError> {
        let mut index_ops = Vec::new();

        loop {
            let next_token = self.next_token(&mut tokens)?;

            let (node, ts) = self.parse_expression(next_token, tokens)?;
            index_ops.push(IndexOperation {
                index: Box::new(node),
            });

            tokens = ts;

            if Ok(Token::Symbol(SymbolToken::SquareClose)) != self.next_token(&mut tokens) {
                return Err(ParsingError::new(
                    ParsingErrorKind::ExpectedNotFound("]".to_string()),
                    self.curr_line,
                ));
            }

            let tok = self.next_token(&mut tokens);
            if Ok(Token::Symbol(SymbolToken::SquareOpen)) == tok {
                continue;
            }

            if let Ok(t) = tok {
                tokens.reinsert(t);
            }
            break;
        }

        let next = self.next_token(&mut tokens);

        if let Ok(Token::Symbol(SymbolToken::Equals)) = next {
            self.parse_assignment(
                Node::Index(IndexNode {
                    initial: Box::new(initial),
                    index_ops,
                }),
                tokens,
            )
        } else {
            if let Ok(tok) = next {
                tokens.reinsert(tok);
            }
            let node = Node::Index(IndexNode {
                initial: Box::new(initial),
                index_ops,
            });

            Ok((node, tokens))
        }
    }

    ///parses anything starting with an ident(ifier)
    pub fn parse_ident<'node, 'text, 'tokens>(
        &mut self,
        identifier: &'node str,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(Node<'node>, TokenStream<'node, 'text, 'tokens>)> {
        let next = self.next_token(&mut tokens);
        match next {
            Err(_) => Ok((
                Node::Identifier {
                    id: identifier,
                    hash: hash(identifier),
                },
                tokens,
            )),
            Ok(Token::Symbol(SymbolToken::RoundOpen)) => self.parse_call(
                Node::Identifier {
                    id: identifier,
                    hash: hash(identifier),
                },
                tokens,
            ),
            Ok(Token::Symbol(SymbolToken::Equals)) => self.parse_assignment(
                Node::Identifier {
                    id: identifier,
                    hash: hash(identifier),
                },
                tokens,
            ),
            Ok(token) => {
                tokens.reinsert(token);
                Ok((
                    Node::Identifier {
                        id: identifier,
                        hash: hash(identifier),
                    },
                    tokens,
                ))
            }
        }
    }

    fn parse_assignment<'node, 'text, 'tokens>(
        &mut self,
        assignee: Node<'node>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(Node<'node>, TokenStream<'node, 'text, 'tokens>)> {
        let next_token = self.next_token(&mut tokens)?;
        let (node, ts) = self.parse_expression(next_token, tokens)?;

        Ok((
            Node::Assignment {
                assignee: Box::new(assignee),
                value: Box::new(node),
            },
            ts,
        ))
    }

    fn parse_call<'node, 'text, 'tokens>(
        &mut self,
        expr: Node<'node>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(Node<'node>, TokenStream<'node, 'text, 'tokens>)> {
        let mut args: Vec<Node<'node>> = vec![];
        loop {
            let next_token = self.next_token(&mut tokens)?;

            let (arg, ts) = self.parse_arg(next_token, tokens)?;
            tokens = ts;
            dbg_print!(&arg);
            if let Some(n) = arg {
                let next = self.next_token(&mut tokens)?;
                match next {
                    Token::Symbol(SymbolToken::Comma) => args.push(n),
                    Token::Symbol(SymbolToken::RoundClose) => {
                        args.push(n);
                        break;
                    }
                    t => {
                        return Err(ParsingError::new(
                            ParsingErrorKind::Unexpected(format!("{:?}", t)),
                            self.curr_line,
                        ))
                    }
                }
            } else {
                break;
            }
        }

        let fcall = Node::FunctionCall {
            expr: Box::new(expr),
            args,
        };
        if let Ok(next) = self.next_token(&mut tokens) {
            if Token::Symbol(SymbolToken::RoundOpen) == next {
                self.parse_call(fcall, tokens)
            } else {
                tokens.reinsert(next);
                Ok((fcall, tokens))
            }
        } else {
            Ok((fcall, tokens))
        }
    }

    pub fn parse_arg<'node, 'text, 'tokens>(
        &mut self,
        token: Token<'tokens>,
        tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(Option<Node<'node>>, TokenStream<'node, 'text, 'tokens>)> {
        match token {
            Token::Symbol(SymbolToken::RoundClose) => Ok((None, tokens)),
            t => {
                let (node, ts) = self.parse_expression(t, tokens)?;
                Ok((Some(node), ts))
            }
        }
    }

    pub fn parse_expression<'node, 'text, 'tokens>(
        &mut self,
        token: Token<'tokens>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(Node<'node>, TokenStream<'node, 'text, 'tokens>)> {
        let (node, ts) = match token {
            Token::Data(data) => Ok((self.parse_data(data), tokens)),
            Token::Identifier(id) => self.parse_ident(id, tokens),
            Token::Keyword(key) => self.parse_keyword(key, tokens),
            t => todo!("error handling {:?}", t),
        }?;

        tokens = ts;

        let next = self.next_token(&mut tokens)?;
        match next {
            Token::Symbol(SymbolToken::SquareOpen) => self.parse_index(node, tokens),
            t => {
                tokens.reinsert(t);

                Ok((node, tokens))
            }
        }
    }

    /// Parses Data out of DataTolkens into DayObjects
    pub fn parse_data<'node>(&mut self, data: DataToken) -> Node<'node> {
        Node::Data(match data {
            DataToken::Integer(i) => DayObject::Integer(i),
            DataToken::Float(f) => DayObject::Float(f),
            DataToken::Bool(b) => DayObject::Bool(b),
            DataToken::Character(c) => DayObject::Character(c),
            DataToken::Str(s) => DayObject::Str(s),
            DataToken::None => DayObject::None,
        })
    }

    fn parse_keyword<'node, 'text, 'tokens>(
        &mut self,
        keyword: KeywordToken,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(Node<'node>, TokenStream<'node, 'text, 'tokens>)> {
        dbg_print!(&keyword);
        match keyword {
            KeywordToken::Ret => {
                let (expr, ts) = self.parse_ret(tokens)?;
                match expr {
                    Some(expr) => Ok((Node::Ret(Some(Arc::new(expr))), ts)),
                    None => Ok((Node::Ret(None), ts)),
                }
            }
            KeywordToken::Let => self.parse_declaration(tokens),
            KeywordToken::Const => self.parse_const_declaration(tokens),
            KeywordToken::If => {
                tokens.reinsert(KeywordToken::If.into());
                let mut branches = vec![];
                while let Ok(next_token) = self.next_token(&mut tokens) {
                    match next_token {
                        Token::Keyword(KeywordToken::If) if branches.len() != 0 => {
                            tokens.reinsert(Token::Keyword(KeywordToken::If));
                            break;
                        }
                        next => {
                            let (b, ts) = self.parse_branch_inner(next, tokens)?;
                            tokens = ts;
                            if let Some(b) = b {
                                branches.push(b)
                            } else {
                                break;
                            }
                        }
                    }
                }

                Ok((Node::BranchNode(branches), tokens))
            }
            KeywordToken::While => {
                let next_token = self.next_token(&mut tokens)?;
                let (condition, mut tokens) = self.parse_expression(next_token, tokens)?;
                dbg_print!(&condition);
                if Token::Symbol(SymbolToken::CurlyOpen) != self.next_token(&mut tokens)? {
                    return Err(ParsingError::new(
                        ParsingErrorKind::ExpectedNotFound("{".to_string()),
                        self.curr_line,
                    ));
                }
                let (block, tokens) = self.parse(tokens, NodePurpose::While)?;
                dbg_print!(&block);

                Ok((
                    Node::While {
                        condition: Box::new(condition),
                        block,
                    },
                    tokens,
                ))
            }
            KeywordToken::Fn => {
                let next = self.next_token(&mut tokens)?;
                let id = if let Token::Identifier(s) = next {
                    Some(s)
                } else {
                    tokens.reinsert(next);
                    None
                };

                if Ok(Token::Symbol(SymbolToken::CurlyOpen)) != self.next_token(&mut tokens) {
                    return Err(ParsingError::new(
                        ParsingErrorKind::ExpectedNotFound("{".to_string()),
                        self.curr_line,
                    ));
                }
                let (block, tokens) = self.parse(tokens, NodePurpose::Function)?;

                Ok((Node::function_decl(id, block), tokens))
            }
            KeywordToken::For => {
                let ident = self.get_identifier(&mut tokens)?;
                if Some(Token::Keyword(KeywordToken::In)) != tokens.next() {
                    return Err(ParsingError::new(
                        ParsingErrorKind::ExpectedNotFound("in".to_string()),
                        self.curr_line,
                    ));
                }
                let next_token = self.next_token(&mut tokens)?;
                let (iter, mut tokens) = self.parse_expression(next_token, tokens)?;
                dbg_print!(&iter);
                if Some(Token::Symbol(SymbolToken::CurlyOpen)) != tokens.next() {
                    return Err(ParsingError::new(
                        ParsingErrorKind::ExpectedNotFound("{".to_string()),
                        self.curr_line,
                    ));
                }
                let (block, tokens) = self.parse(tokens, NodePurpose::For)?;
                dbg_print!(&block);
                Ok((
                    Node::For {
                        ident,
                        hash: hash(ident),
                        expr: Box::new(iter),
                        block,
                    },
                    tokens,
                ))
            }
            _ => todo!(),
        }
    }

    fn parse_ret<'node, 'text, 'tokens>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(Option<Node<'node>>, TokenStream<'node, 'text, 'tokens>)> {
        match self.next_token(&mut tokens) {
            Err(_) => Ok((None, tokens)),
            Ok(t) => {
                let expr = self.parse_expression(t, tokens)?;
                Ok((Some(expr.0), expr.1))
            }
        }
    }

    /// Parses the branch belonging to the token `tok`
    ///
    /// ### Returns:
    /// A Result with either a ParsingError or a tuple containing:
    /// 1. Option<BranchNode>:
    ///     - *Some* if branch could be passed
    ///     - *None* if no branch could belong to `tok`
    /// 2. TokenStream
    fn parse_branch_inner<'node, 'text, 'tokens>(
        &mut self,
        tok: Token<'tokens>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(
        Option<BranchNode<'node>>,
        TokenStream<'node, 'text, 'tokens>,
    )> {
        if let Token::Keyword(ktok) = tok {
            match ktok {
                KeywordToken::If => {
                    let (condition, block, tokens) = self.parse_if_inner(tokens)?;
                    Ok((Some(BranchNode::If { condition, block }), tokens))
                }
                KeywordToken::Else => match self.next_token(&mut tokens)? {
                    Token::Keyword(KeywordToken::If) => {
                        let (condition, block, tokens) = self.parse_if_inner(tokens)?;
                        Ok((Some(BranchNode::ElseIf { condition, block }), tokens))
                    }
                    Token::Symbol(SymbolToken::CurlyOpen) => {
                        let (block, tokens) = self.parse(tokens, NodePurpose::Conditional)?;
                        Ok((Some(BranchNode::Else { block }), tokens))
                    }
                    t => {
                        // TODO Display token prettier
                        return Err(ParsingError::new(
                            ParsingErrorKind::Unexpected(format!("{:?}", t)),
                            self.curr_line,
                        ));
                    }
                },
                _ => {
                    // No branch to parse
                    tokens.reinsert(ktok.into());
                    Ok((None, tokens))
                }
            }
        } else {
            tokens.reinsert(tok.into());

            Ok((None, tokens))
        }
    }

    fn parse_if_inner<'node, 'text, 'tokens>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<(
        Box<Node<'node>>,
        RootNode<'node>,
        TokenStream<'node, 'text, 'tokens>,
    )> {
        let next_token = self.next_token(&mut tokens)?;

        let (condition, mut tokens) = self.parse_expression(next_token, tokens)?;

        if Ok(Token::Symbol(SymbolToken::CurlyOpen)) != self.next_token(&mut tokens) {
            return Err(ParsingError::new(
                ParsingErrorKind::ExpectedNotFound("{".to_string()),
                self.curr_line,
            ));
        }

        let (block, tokens) = self.parse(tokens, NodePurpose::Conditional)?;

        Ok((Box::new(condition), block, tokens))
    }

    fn parse_declaration<'node, 'text, 'tokens>(
        &mut self,
        tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> Result<(Node<'node>, TokenStream<'node, 'text, 'tokens>), ParsingError> {
        let decl = self.decl_inner(tokens)?;
        Ok((
            Node::Declaration {
                id: decl.0,
                hash: hash(decl.0),
                value: decl.1,
            },
            decl.2,
        ))
    }

    fn parse_const_declaration<'node, 'text, 'tokens>(
        &mut self,
        tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> Result<(Node<'node>, TokenStream<'node, 'text, 'tokens>), ParsingError> {
        let decl = self.decl_inner(tokens)?;
        Ok((
            Node::ConstDeclaration {
                id: decl.0,
                hash: hash(decl.0),
                value: decl.1,
            },
            decl.2,
        ))
    }

    fn decl_inner<'node, 'text, 'tokens>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> Result<
        (
            &'node str,
            Box<Node<'node>>,
            TokenStream<'node, 'text, 'tokens>,
        ),
        ParsingError,
    > {
        //NOTE currently testing this little cool macro
        let id =
            expect!(self.next_token(&mut tokens)? => Token::Identifier | "Expected identifier");
        if Ok(Token::Symbol(SymbolToken::Equals)) != self.next_token(&mut tokens) {
            return Err(ParsingError::new(
                ParsingErrorKind::ExpectedNotFound("=".to_string()),
                self.curr_line,
            ));
        }
        let next_token = self.next_token(&mut tokens)?;
        let (node, ts) = self.parse_expression(next_token, tokens)?;

        Ok((id, Box::new(node), ts))
    }

    /// Retruns the next non-meta token
    /// And handles the meta-tokens,
    /// by e.g. incrementing line numbers.
    /// ### Errors
    /// `UnexpectedEnd` when no more tokens are in the token stream (`tokens.next()` returns `None`)
    fn next_token<'node, 'text, 'tokens>(
        &mut self,
        tokens: &mut TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<Token<'tokens>> {
        loop {
            match tokens.next() {
                Some(Token::Newline) => {
                    self.curr_line += 1;
                }
                Some(other_token) => {
                    return Ok(other_token);
                }
                None => {
                    return Err(ParsingError::new(
                        ParsingErrorKind::UnexpectedEndOfInput,
                        self.curr_line,
                    ))
                }
            }
        }
    }

    fn get_identifier<'node, 'text, 'tokens>(
        &mut self,
        tokens: &mut TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<&'text str> {
        if let Token::Identifier(id) = self.next_token(tokens)? {
            Ok(id)
        } else {
            Err(ParsingError::new(
                ParsingErrorKind::ExpectedNotFound("identifier".to_string()),
                self.curr_line,
            ))
        }
    }
}
