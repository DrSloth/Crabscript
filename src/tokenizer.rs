use regex_lexer::{LexerBuilder, Lexer};

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Data(DataToken),
    //Operator(OperatorToken),
    //Control(ControlToken),
    Identifier(&'a str),
    //Null,
    Symbol(SymbolToken),
}

impl From<DataToken> for Token<'_> {
    fn from(data: DataToken)->Self{
        Token::Data(data)
    }
}

impl From<SymbolToken> for Token<'_> {
    fn from(bracket: SymbolToken)->Self{
        Token::Symbol(bracket)
    }
}


#[derive(Debug, PartialEq)]
pub enum DataToken {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Character(char),
    Str(String),
}
impl std::cmp::Eq for DataToken{

}

#[derive(Debug, PartialEq, Eq)]
pub enum SymbolToken {
    RoundOpen,
    RoundClose,
    DeclarationOperator,
    AssignmentOperator,
    Comma,
}
/*
pub enum OperatorToken {
    Puls,
    Minus,
    Astrist,
    Slash,
    Percent,
}



pub enum ControlToken {
    If,
    Else,
    Elif,
    While,
}*/

pub fn build_lexer<'t>() -> Result<Lexer<'t, Token<'t>>, regex::Error> {
LexerBuilder::new()

.token(r"(true|false)", |tok| Some(DataToken::Bool(tok.parse().unwrap()).into()))
.token(r"-?[0-9]+", |tok| Some(DataToken::Integer(tok.parse().unwrap()).into()))
.token(r"-?[0-9]+/.[0-9]+", |tok| Some(DataToken::Float(tok.parse().unwrap()).into()))
.token(r"'.'", |tok| Some(DataToken::Character(tok.parse().unwrap()).into()))
.token("\".*\"", |tok| Some(DataToken::Str(tok[1..tok.len() - 1].parse().unwrap()).into()))

.token(r"\(", |_| Some(SymbolToken::RoundOpen.into()))
.token(r"\)", |_| Some(SymbolToken::RoundClose.into()))
.token(r",", |_| Some(SymbolToken::Comma.into()))

.token(r"(_|[a-zA-Z])[a-zA-Z_0-9]*", |tok| Some(Token::Identifier(tok)))

/*
.token(r"\+", |tok| Some(Token::Operator::Puls(tok.parse().unwrap()))
.token(r"-", |tok| Some(Token::Operator::Minus(tok.parse().unwrap()))
.token(r"\*", |tok| Some(Token::Operator::Atrist(tok.parse().unwrap()))
.token(r"/", |tok| Some(Token::Operator::Slash(tok.parse().unwrap()))
.token(r"%" |tok| Some(Token::Operator::(tok.parse().unwrap()))
*/

.token(r"\s", |_| None)

.build()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world(){
            let source = "print(\"Hello, World!\") ";
            assert_eq!(
                build_lexer().unwrap().tokens(source).collect::<Vec<_>>(),
                vec![
                    Token::Identifier("print"), SymbolToken::RoundOpen.into(), DataToken::Str("Hello, World!".to_string()).into(), SymbolToken::RoundClose.into()
                ],
            )
    }
}