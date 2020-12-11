use super::parsing_error::{ParsingError, ParsingErrorKind, ParsingResult};
use super::PreMap;
use crate::{
    base::DayObject,
    manager::RuntimeManager,
    node::*,
    tokenizer::{DataToken, KeywordToken, SymbolToken, Token, TokenStream},
};
use std::sync::Arc;

//IMPORTANT
//NOTE For now the var map will be preallocated instead of resolving at runtime
//NOTE For now the var map will be preallocated instead of resolving at runtime
//NOTE At first this will do the line keeping logic twice but that could be easily optimizable
//NOTE A scope stack will also be used inside the parser. The main complicated thing by now
//is how to solve inner and outer scopes that will be done later
//NOTE is implemented with the indextree map so it could be done multithreaded it's not implemented with
//refcell because it imposes overhead and is harder to implement
//NOTE Some clones might be possible to be saved

//TODO Make constants matter

pub struct Parser<'tokens> {
    curr_line: u64,
    pre_map: PreMap,
    var_tree: VarTree<'tokens>,
}

impl<'tokens> Parser<'tokens> {
    pub fn new(pre_map: PreMap) -> Self {
        Parser {
            curr_line: 1,
            pre_map,
            var_tree: VarTree::new(),
        }
    }

    pub fn parse_tokens<'node, 'text>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<Block> {
        tokens = TokenStream::from(self.fill_var_map(tokens)?);
        dbg_print!(format!("{:#?}", &self.var_tree));
        let blk = self.parse(tokens, NodePurpose::TopLevel, None);
        dbg_print_pretty!(blk);
        blk.map(|b| b.0)
    }

    pub fn fill_var_map<'node, 'text>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
    ) -> ParsingResult<Vec<Token<'tokens>>> {
        let mut v = Vec::with_capacity(tokens.size_hint());

        while let Some(t) = tokens.next() {
            match &t {
                Token::Keyword(KeywordToken::Let)
                | Token::Keyword(KeywordToken::Const)
                | Token::Keyword(KeywordToken::Fn) => {
                    v.push(t);
                    let idt = self.next_token(&mut tokens)?;
                    if let Token::Identifier(ident) = idt {
                        v.push(idt);
                        let depth = self.var_tree.depth();
                        let vars = self.var_tree.get_current_mut();
                        dbg_print_pretty!(v);
                        vars.insert(
                            ident,
                            Variable {
                                depth,
                                id: vars.len(),
                            },
                        );
                    } else if Token::Symbol(SymbolToken::CurlyOpen) == idt {
                        self.var_tree.to_new_successor();
                        self.var_tree.pre_order.push(self.var_tree.current);
                        v.push(idt);
                    }
                }
                Token::Keyword(KeywordToken::For) => {
                    v.push(t);
                    let idt = self.next_token(&mut tokens)?;
                    if let Token::Identifier(ident) = idt {
                        v.push(idt);
                        loop {
                            let tok = self.next_token(&mut tokens)?;
                            if tok == Token::Symbol(SymbolToken::CurlyOpen) {
                                v.push(tok);
                                break;
                            }
                            v.push(tok);
                        }
                        self.var_tree.to_new_successor();
                        self.var_tree.pre_order.push(self.var_tree.current);
                        let depth = self.var_tree.depth();
                        let vars = self.var_tree.get_current_mut();
                        vars.insert(ident, Variable { depth, id: 0 });
                    }
                }
                Token::Symbol(SymbolToken::CurlyOpen) => {
                    self.var_tree.to_new_successor();
                    self.var_tree.pre_order.push(self.var_tree.current);
                    v.push(t)
                }
                Token::Symbol(SymbolToken::CurlyClose) => {
                    self.var_tree.to_predecessor();
                    v.push(t)
                }
                Token::Newline => {
                    self.curr_line += 1;
                    v.push(t)
                }
                _ => v.push(t),
            }
        }

        dbg_print_pretty!(v);

        self.curr_line = 0;

        Ok(v)
    }

    fn parse<'node, 'text>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        purpose: NodePurpose,
        predecessor: Option<Arc<RuntimeManager>>,
    ) -> ParsingResult<(Block, TokenStream<'node, 'text, 'tokens>)> {
        let mut block =
            Block::new_capacity_predecessor(purpose, self.var_tree.len_vars(), predecessor);

        self.var_tree.to_next_preorder();
        let current = self.var_tree.current;

        while let Ok(token) = self.next_token(&mut tokens) {
            dbg_print!(&token);
            self.var_tree.current = current;
            match token {
                Token::Keyword(k) => {
                    let (node, ts) = self.parse_keyword(k, tokens, Arc::clone(&block.scope))?;
                    tokens = ts;
                    block.push(node)
                }
                Token::Data(val) => block.push(self.parse_data(val)),

                Token::Identifier(id) => {
                    let (node, ts) = self.parse_ident(id, tokens, Arc::clone(&block.scope))?;
                    tokens = ts;
                    block.push(node)
                }

                Token::Symbol(sym) => match sym {
                    SymbolToken::SquareOpen => {
                        let (node, ts) = self.parse_index(
                            match block.pop() {
                                Some(n) => n,
                                None => {
                                    return Err(ParsingError::new(
                                        ParsingErrorKind::UnexpectedEndOfInput,
                                        self.curr_line,
                                    ))
                                }
                            },
                            tokens,
                            Arc::clone(&block.scope),
                        )?;

                        block.push(node);
                        tokens = ts
                    }
                    SymbolToken::CurlyOpen => {
                        let (node, ts) =
                            self.parse(tokens, NodePurpose::Block, Some(Arc::clone(&block.scope)))?;
                        tokens = ts;
                        block.push(Node::Block(node))
                    }
                    SymbolToken::CurlyClose => {
                        if let NodePurpose::TopLevel = block.block.purpose {
                            return Err(ParsingError::unexpected(self.curr_line, "}".to_string()));
                        } else {
                            return Ok((block, tokens));
                        }
                    }
                    SymbolToken::RoundOpen => {
                        let expr = block.pop().ok_or(ParsingError::new(
                            ParsingErrorKind::ExpectedNotFound("Preceeding function".to_string()),
                            self.curr_line,
                        ))?;
                        let (node, ts) = self.parse_call(expr, tokens, Arc::clone(&block.scope))?;
                        tokens = ts;
                        block.push(node);
                    }
                    t => return Err(ParsingError::unexpected(self.curr_line, format!("{:?}", t))),
                },
                Token::Newline => {
                    dbg_print!(self.curr_line);
                    self.curr_line += 1
                }
            }
            //dbg_print!(&block);
        }
        dbg_print_pretty!(block);
        Ok((block, tokens))
    }

    pub fn parse_index<'node, 'text>(
        &mut self,
        initial: Node,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> Result<(Node, TokenStream<'node, 'text, 'tokens>), ParsingError> {
        let mut index_ops = Vec::new();

        loop {
            let next_token = self.next_token(&mut tokens)?;

            let (node, ts) = self.parse_expression(next_token, tokens, Arc::clone(&predecessor))?;
            index_ops.push(IndexOperation {
                index: Box::new(node),
            });

            tokens = ts;

            let tok = self.next_token(&mut tokens);
            if Ok(Token::Symbol(SymbolToken::SquareClose)) != tok {
                return Err(ParsingError::unexpected_expected(
                    self.curr_line,
                    format!("{:?}", tok),
                    "]".to_string(),
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
                predecessor,
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

    //TODO Implement the get_ident method returning either an RustFunction or an Variable Position
    //TODO Change the current approach to one with Unresolved Nodes to be more friendly with the interactive shell

    fn get_ident<'node>(&mut self, identifier: &'node str) -> Option<Node> {
        /* if identifier == "args" {
            return Some(Node::Identifier(IdentifierNode::Args));
        } */

        if let Some(pref) = self.pre_map.get(identifier) {
            return Some(Node::RustFunction(ConstRustFn(pref.clone())));
        }

        for i in self.var_tree.current.ancestors(&self.var_tree.arena) {
            if let Some(v) = self.var_tree.arena.get(i) {
                if let Some(v) = v.get().get(identifier) {
                    //println!("var {:?}", v);
                    return Some(Node::Identifier(IdentifierNode::new(v.id, v.depth)));
                }
            }
        }

        None
    }

    ///parses anything starting with an ident(ifier)
    pub fn parse_ident<'node, 'text>(
        &mut self,
        identifier: &'node str,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Node, TokenStream<'node, 'text, 'tokens>)> {
        let next = self.next_token(&mut tokens);
        //TODO better error message
        let ident = self
            .get_ident(identifier)
            .unwrap_or_else(|| panic!("Access to undefined ident {}", identifier));
        match next {
            Err(_) => Ok((ident, tokens)),
            Ok(Token::Symbol(SymbolToken::RoundOpen)) => {
                self.parse_call(ident, tokens, predecessor)
            }
            Ok(Token::Symbol(SymbolToken::Equals)) => {
                self.parse_assignment(ident, tokens, predecessor)
            }
            Ok(token) => {
                tokens.reinsert(token);
                Ok((ident, tokens))
            }
        }
    }

    fn parse_assignment<'node, 'text>(
        &mut self,
        assignee: Node,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Node, TokenStream<'node, 'text, 'tokens>)> {
        let next_token = self.next_token(&mut tokens)?;
        let (node, ts) = self.parse_expression(next_token, tokens, predecessor)?;

        Ok((
            Node::Assignment {
                assignee: Box::new(assignee),
                value: Box::new(node),
            },
            ts,
        ))
    }

    fn parse_call<'node, 'text>(
        &mut self,
        expr: Node,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Node, TokenStream<'node, 'text, 'tokens>)> {
        let mut args: Vec<Node> = vec![];
        loop {
            let next_token = self.next_token(&mut tokens)?;

            let (arg, ts) = self.parse_arg(next_token, tokens, Arc::clone(&predecessor))?;
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
                        return Err(ParsingError::unexpected_expected(
                            self.curr_line,
                            format!("{:?}", t),
                            ", or )".to_string(),
                        ));
                    }
                }
            } else {
                break;
            }
        }

        let fcall = Node::FunctionCall(
            FunctionCallNode {
                expr: Box::new(expr),
                args,
                arg_cache: Default::default()
            }
        );
        if let Ok(next) = self.next_token(&mut tokens) {
            if Token::Symbol(SymbolToken::RoundOpen) == next {
                self.parse_call(fcall, tokens, predecessor)
            } else {
                tokens.reinsert(next);
                Ok((fcall, tokens))
            }
        } else {
            Ok((fcall, tokens))
        }
    }

    pub fn parse_arg<'node, 'text>(
        &mut self,
        token: Token<'tokens>,
        tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Option<Node>, TokenStream<'node, 'text, 'tokens>)> {
        match token {
            Token::Symbol(SymbolToken::RoundClose) => Ok((None, tokens)),
            t => {
                let (node, ts) = self.parse_expression(t, tokens, predecessor)?;
                Ok((Some(node), ts))
            }
        }
    }

    pub fn parse_expression<'node, 'text>(
        &mut self,
        token: Token<'tokens>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Node, TokenStream<'node, 'text, 'tokens>)> {
        let (node, ts) = match token {
            Token::Data(data) => Ok((self.parse_data(data), tokens)),
            Token::Identifier(id) => self.parse_ident(id, tokens, Arc::clone(&predecessor)),
            Token::Keyword(key) => self.parse_keyword(key, tokens, Arc::clone(&predecessor)),
            t => todo!("error handling {:?}", t),
        }?;

        tokens = ts;

        let next = self.next_token(&mut tokens)?;
        match next {
            Token::Symbol(SymbolToken::SquareOpen) => self.parse_index(node, tokens, predecessor),
            t => {
                dbg_print_pretty!(t);
                tokens.reinsert(t);
                dbg_print_pretty!(tokens);

                Ok((node, tokens))
            }
        }
    }

    /// Parses Data out of DataTolkens into DayObjects
    pub fn parse_data(&mut self, data: DataToken) -> Node {
        Node::Data(match data {
            DataToken::Integer(i) => DayObject::Integer(i),
            DataToken::Float(f) => DayObject::Float(f),
            DataToken::Bool(b) => DayObject::Bool(b),
            DataToken::Character(c) => DayObject::Character(c),
            DataToken::Str(s) => DayObject::Str(s),
            DataToken::None => DayObject::None,
        })
    }

    fn parse_keyword<'node, 'text>(
        &mut self,
        keyword: KeywordToken,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Node, TokenStream<'node, 'text, 'tokens>)> {
        dbg_print!(&keyword);
        match keyword {
            KeywordToken::Ret => {
                let (expr, ts) = self.parse_ret(tokens, predecessor)?;
                match expr {
                    Some(expr) => Ok((Node::Ret(Some(Arc::new(expr))), ts)),
                    None => Ok((Node::Ret(None), ts)),
                }
            }
            KeywordToken::Let => self.parse_declaration(tokens, predecessor),
            KeywordToken::Const => self.parse_const_declaration(tokens, predecessor),
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
                            let (b, ts) =
                                self.parse_branch_inner(next, tokens, Arc::clone(&predecessor))?;
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
                let (condition, mut tokens) =
                    self.parse_expression(next_token, tokens, Arc::clone(&predecessor))?;
                dbg_print!(&condition);
                if Token::Symbol(SymbolToken::CurlyOpen) != self.next_token(&mut tokens)? {
                    return Err(ParsingError::new(
                        ParsingErrorKind::ExpectedNotFound("{".to_string()),
                        self.curr_line,
                    ));
                }
                let (block, tokens) = self.parse(tokens, NodePurpose::While, Some(predecessor))?;
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
                let (block, tokens) =
                    self.parse(tokens, NodePurpose::Function, Some(predecessor))?;

                Ok((Node::function_decl(block, id == None), tokens))
            }
            KeywordToken::For => {
                self.get_identifier(&mut tokens)?;
                if Some(Token::Keyword(KeywordToken::In)) != tokens.next() {
                    return Err(ParsingError::new(
                        ParsingErrorKind::ExpectedNotFound("in".to_string()),
                        self.curr_line,
                    ));
                }
                let next_token = self.next_token(&mut tokens)?;
                let (iter, mut tokens) =
                    self.parse_expression(next_token, tokens, Arc::clone(&predecessor))?;
                dbg_print!(&iter);
                if Some(Token::Symbol(SymbolToken::CurlyOpen)) != tokens.next() {
                    return Err(ParsingError::new(
                        ParsingErrorKind::ExpectedNotFound("{".to_string()),
                        self.curr_line,
                    ));
                }
                let (block, tokens) = self.parse(tokens, NodePurpose::For, Some(predecessor))?;
                dbg_print!(&block);
                Ok((
                    Node::For {
                        expr: Box::new(iter),
                        block,
                    },
                    tokens,
                ))
            }
            _ => todo!(),
        }
    }

    fn parse_ret<'node, 'text>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Option<Node>, TokenStream<'node, 'text, 'tokens>)> {
        match self.next_token(&mut tokens) {
            Err(_) => Ok((None, tokens)),
            Ok(t) => {
                let expr = self.parse_expression(t, tokens, predecessor)?;
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
    fn parse_branch_inner<'node, 'text>(
        &mut self,
        tok: Token<'tokens>,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Option<BranchNode>, TokenStream<'node, 'text, 'tokens>)> {
        if let Token::Keyword(ktok) = tok {
            match ktok {
                KeywordToken::If => {
                    let (condition, block, tokens) = self.parse_if_inner(tokens, predecessor)?;
                    let block = IfBlock { condition, block };
                    Ok((Some(BranchNode::ElseIf {block}), tokens))
                }
                KeywordToken::Else => match self.next_token(&mut tokens)? {
                    Token::Keyword(KeywordToken::If) => {
                        let (condition, block, tokens) =
                            self.parse_if_inner(tokens, predecessor)?;
                        let block = IfBlock { condition, block };
                        Ok((Some(BranchNode::ElseIf {block}), tokens))
                    }
                    Token::Symbol(SymbolToken::CurlyOpen) => {
                        let (block, tokens) =
                            self.parse(tokens, NodePurpose::Conditional, Some(predecessor))?;
                        Ok((Some(BranchNode::Else { block }), tokens))
                    }
                    t => {
                        return Err(ParsingError::unexpected_expected(
                            self.curr_line,
                            format!("{:?}", t),
                            "if or {".to_string(),
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

    fn parse_if_inner<'node, 'text>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> ParsingResult<(Box<Node>, Block, TokenStream<'node, 'text, 'tokens>)> {
        let next_token = self.next_token(&mut tokens)?;

        let (condition, mut tokens) =
            self.parse_expression(next_token, tokens, Arc::clone(&predecessor))?;

        if Ok(Token::Symbol(SymbolToken::CurlyOpen)) != self.next_token(&mut tokens) {
            return Err(ParsingError::new(
                ParsingErrorKind::ExpectedNotFound("{".to_string()),
                self.curr_line,
            ));
        }

        let (block, tokens) = self.parse(tokens, NodePurpose::Conditional, Some(predecessor))?;

        Ok((Box::new(condition), block, tokens))
    }

    fn parse_declaration<'node, 'text>(
        &mut self,
        tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> Result<(Node, TokenStream<'node, 'text, 'tokens>), ParsingError> {
        let decl = self.decl_inner(tokens, predecessor)?;
        Ok((Node::Declaration { value: decl.1 }, decl.2))
    }

    fn parse_const_declaration<'node, 'text>(
        &mut self,
        tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> Result<(Node, TokenStream<'node, 'text, 'tokens>), ParsingError> {
        let decl = self.decl_inner(tokens, predecessor)?;
        Ok((Node::ConstDeclaration { value: decl.1 }, decl.2))
    }

    fn decl_inner<'node, 'text>(
        &mut self,
        mut tokens: TokenStream<'node, 'text, 'tokens>,
        predecessor: Arc<RuntimeManager>,
    ) -> Result<(&'node str, Box<Node>, TokenStream<'node, 'text, 'tokens>), ParsingError> {
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
        let (node, ts) = self.parse_expression(next_token, tokens, predecessor)?;

        Ok((id, Box::new(node), ts))
    }

    /// Retruns the next non-meta token
    /// And handles the meta-tokens,
    /// by e.g. incrementing line numbers.
    /// ### Errors
    /// `UnexpectedEnd` when no more tokens are in the token stream (`tokens.next()` returns `None`)
    fn next_token<'node, 'text>(
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

    fn get_identifier<'node, 'text>(
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

use indextree::{Arena, NodeId};
use std::collections::HashMap;
type Scope<'a> = HashMap<&'a str, Variable>;

//TODO Save all NodeIds in an Preorder ordering and traverse it by that

#[derive(Debug)]
struct VarTree<'a> {
    arena: Arena<Scope<'a>>,
    current: NodeId,
    pre_order: Vec<NodeId>,
}

impl<'a> VarTree<'a> {
    fn new() -> Self {
        let mut arena = Arena::default();
        let current = arena.new_node(Default::default());
        VarTree {
            arena,
            current,
            pre_order: vec![current],
        }
    }

    fn to_predecessor(&mut self) {
        self.current = self.predecessor().unwrap_or(self.current);
    }

    fn predecessor(&self) -> Option<NodeId> {
        self.current.ancestors(&self.arena).skip(1).next()
    }

    fn to_new_successor(&mut self) {
        self.current = self.new_successor();
    }

    fn new_successor(&mut self) -> NodeId {
        let new = self.arena.new_node(Default::default());
        self.current.append(new, &mut self.arena);
        new
    }

    fn get_current(&self) -> &Scope<'a> {
        self.arena.get(self.current).unwrap().get()
    }

    fn get_current_mut(&mut self) -> &mut Scope<'a> {
        self.arena.get_mut(self.current).unwrap().get_mut()
    }

    fn depth(&self) -> usize {
        self.current.ancestors(&self.arena).skip(1).count()
    }

    fn len_vars(&self) -> usize {
        self.get_current().len()
    }

    fn to_next_preorder(&mut self) {
        self.current = self.get_next_preorder();
        //println!("l{} | c{} | v{:?}", self.pre_order.len(), self.current, self.get_current());
    }

    fn get_next_preorder(&mut self) -> NodeId {
        self.pre_order.remove(0)
    }
}

#[derive(Debug, Clone)]
struct Variable {
    id: usize,
    depth: usize,
}
