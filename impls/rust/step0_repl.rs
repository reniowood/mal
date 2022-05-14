use std::io::{self, BufRead, Write};

fn main() {
    loop {
        print!("user> ");
        if let Err(error) = io::stdout().flush() {
            println!("{}", error);
            return;
        }

        let mut line = String::new();
        if let Ok(0) = io::stdin().lock().read_line(&mut line) {
            return;
        }

        println!("{}", rep(&line));
    }
}

fn rep(input: &str) -> &str {
    print(eval(read(input)))
}

fn read(input: &str) -> &str {
    input
}

fn eval(input: &str) -> &str {
    input
}

fn print(input: &str) -> &str {
    input
}