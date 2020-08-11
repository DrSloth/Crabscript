mod tokenizer;
fn main() {
    for el in tokenizer::build_lexer().unwrap().tokens("print(\"Hello, World!\") "){
        println!("{:?}", el);
    }
}
