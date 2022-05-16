use printer::pr_str;
use reader::read_str;
use rustyline::Editor;
use types::MalType;

mod reader;
mod types;
mod printer;

fn main() {
    let mut rl = Editor::<()>::new();

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match rep(&line) {
                    Ok(result) => println!("{}", result),
                    Err(message) => eprintln!("Error: {}", message),
                }
            }
            Err(_) => break
        };
    }
}

fn rep(input: &str) -> Result<String, String> {
    match read(input) {
        Ok(value) => Ok(print(eval(&value))),
        Err(message) => Err(message)
    }
}

fn read(input: &str) -> Result<MalType, String> {
    read_str(input)
}

fn eval(input: &MalType) -> &MalType {
    input
}

fn print(input: &MalType) -> String {
    pr_str(input)
}