use std::io;

fn main() {
    let pattern_string: String = std::env::args()
        .nth(1)
        .expect("missing required command-line argument: <pattern>");
    let pattern: &String = &pattern_string;
    let mut input: String = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(_n) => {
                if input.is_empty() {
                    break;
                }
                if input.contains(pattern) {
                    println!("{}", input.replace('\n', ""));
                }
                input.clear();
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
