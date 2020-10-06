use one_day::run;

fn main() {
    let file_content = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    run(&file_content);
}
