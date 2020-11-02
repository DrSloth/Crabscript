use regex_lexer::{Lexer, LexerBuilder, Tokens};

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Data(DataToken),
    //Operator(OperatorToken),
    Keyword(KeywordToken),
    Identifier(&'a str),
    //Null,
    Symbol(SymbolToken),
    Newline,
}

impl From<DataToken> for Token<'_> {
    fn from(data: DataToken) -> Self {
        Token::Data(data)
    }
}

impl From<SymbolToken> for Token<'_> {
    fn from(bracket: SymbolToken) -> Self {
        Token::Symbol(bracket)
    }
}

impl From<KeywordToken> for Token<'_> {
    fn from(key: KeywordToken) -> Self {
        Token::Keyword(key)
    }
}

#[derive(Debug, PartialEq)]
pub enum DataToken {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Character(char),
    Str(String),
    None,
}

impl std::cmp::Eq for DataToken {}

#[derive(Debug, PartialEq, Eq)]
pub enum SymbolToken {
    RoundOpen,
    RoundClose,
    Equals,
    CurlyOpen,
    CurlyClose,
    SquareOpen,
    SquareClose,
    Comma,
    Semicolon,
}

/*
pub enum OperatorToken {
    Puls,
    Minus,
    Astrist,
    Slash,
    Percent,
}
*/

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum KeywordToken {
    If,
    Else,
    Elif,
    While,
    Let,
    Fn,
    Const,
    Ret,
    For,
    In,
}

pub fn build_lexer<'t>() -> Result<Lexer<'t, Token<'t>>, regex::Error> {
    LexerBuilder::new()
        .token("=", |_| Some(SymbolToken::Equals.into()))
        .token(r"-?[0-9]+", |tok| {
            Some(DataToken::Integer(tok.parse().unwrap()).into())
        })
        .token(r"-?[0-9]+\.[0-9]+", |tok| {
            Some(DataToken::Float(tok.parse().unwrap()).into())
        })
        .token(r"'.'", |tok| {
            Some(DataToken::Character(tok[1..tok.len() - 1].parse().unwrap()).into())
        })
        .token("\".*?\"", |tok| {
            let s = Some(DataToken::Str(tok[1..tok.len() - 1].replace("\\n", "\n")).into());
            s
        })
        .token(r"\(", |_| Some(SymbolToken::RoundOpen.into()))
        .token(r"\)", |_| Some(SymbolToken::RoundClose.into()))
        .token(r"\{", |_| Some(SymbolToken::CurlyOpen.into()))
        .token(r"\}", |_| Some(SymbolToken::CurlyClose.into()))
        .token(r"\[", |_| Some(SymbolToken::SquareOpen.into()))
        .token(r"\]", |_| Some(SymbolToken::SquareClose.into()))
        .token(r",", |_| Some(SymbolToken::Comma.into()))
        .token(r";", |_| Some(SymbolToken::Semicolon.into()))
        .token(r"(_|[a-zA-Z])[a-zA-Z_0-9]*", |tok| {
            Some(Token::Identifier(tok))
        })
        .token(r"//.*?\n", |_| Some(Token::Newline))
        .token(r"(true|false)", |tok| {
            Some(DataToken::Bool(tok.parse().unwrap()).into())
        })
        .token("ret", |_| Some(KeywordToken::Ret.into()))
        .token("while", |_| Some(KeywordToken::While.into()))
        .token("if", |_| Some(KeywordToken::If.into()))
        .token("else", |_| Some(KeywordToken::Else.into()))
        .token("elif", |_| Some(KeywordToken::Elif.into()))
        .token("const", |_| Some(KeywordToken::Const.into()))
        .token("for", |_| Some(KeywordToken::For.into()))
        .token("in", |_| Some(KeywordToken::In.into()))
        //Change to data
        .token("none", |_| Some(DataToken::None.into()))
        .token("let", |_| Some(KeywordToken::Let.into()))
        .token("fn", |_| Some(KeywordToken::Fn.into()))
        /*
        .token(r"\+", |tok| Some(Token::Operator::Puls(tok.parse().unwrap()))
        .token(r"-", |tok| Some(Token::Operator::Minus(tok.parse().unwrap()))
        .token(r"\*", |tok| Some(Token::Operator::Atrist(tok.parse().unwrap()))
        .token(r"/", |tok| Some(Token::Operator::Slash(tok.parse().unwrap()))
        .token(r"%" |tok| Some(Token::Operator::(tok.parse().unwrap()))
        */
        .token(r"\s", |_| None)
        .token(r"//.*", |_| Some(Token::Newline))
        .token("\n", |_| Some(Token::Newline))
        .build()
}

#[derive(Debug)]
pub struct TokenStream<'node, 'text, 'tokens> {
    untouched_tokens: Option<Tokens<'node, 'text, Token<'tokens>>>,
    peeked_tokens: Vec<Token<'tokens>>,
}

impl<'node, 'text, 'tokens> TokenStream<'node, 'text, 'tokens> {
    pub fn new(tokens: Tokens<'node, 'text, Token<'tokens>>) -> Self {
        Self {
            untouched_tokens: Some(tokens),
            peeked_tokens: vec![],
        }
    }

    pub fn next(&mut self) -> Option<Token<'tokens>> {
        if self.peeked_tokens.is_empty() {
            if let Some(untouched) = &mut self.untouched_tokens {
                untouched.next()
            } else {
                None
            }
        } else {
            Some(self.peeked_tokens.remove(0))
        }
    }

    pub fn reinsert(&mut self, token: Token<'tokens>) {
        self.peeked_tokens.insert(0, token)
    }

    pub fn size_hint(&self) -> usize {
        if let Some(untouched) = &self.untouched_tokens {
            untouched.size_hint().1.unwrap_or(0) + self.peeked_tokens.len()
        } else {
            self.peeked_tokens.len()
        }
    }

    pub fn into_iter(self) -> Box<dyn Iterator<Item = Token<'tokens>> + 'node> {
        if let Some(untouched) = self.untouched_tokens {
            Box::new(untouched.chain(self.peeked_tokens.into_iter()))
        } else {
            Box::new(self.peeked_tokens.into_iter())
        }
    }
}

impl<'node, 'text, 'tokens> From<Vec<Token<'tokens>>> for TokenStream<'node, 'text, 'tokens> {
    fn from(v: Vec<Token<'tokens>>) -> Self {
        Self {
            untouched_tokens: None,
            peeked_tokens: v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        let source = "print(\"Hello, World!\") ";
        assert_eq!(
            build_lexer().unwrap().tokens(source).collect::<Vec<_>>(),
            vec![
                Token::Identifier("print"),
                SymbolToken::RoundOpen.into(),
                DataToken::Str("Hello, World!".to_string()).into(),
                SymbolToken::RoundClose.into()
            ],
        )
    }
}
