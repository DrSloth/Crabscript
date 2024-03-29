use crabscript::run;

fn main() {
    let file_content = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    if let Err(e) = run(&file_content) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
