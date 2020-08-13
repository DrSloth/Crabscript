use one_day::{tokenizer, parser};

fn main() {
    let lexer = tokenizer::build_lexer().unwrap();
    let tokens = lexer.tokens("print(\"Hello, World!\") ");
    let nodes = parser::parse(tokens);
    dbg!(&nodes);
    for n in nodes {
        n.execute();
    }
}
