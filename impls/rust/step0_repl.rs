use rustyline::Editor;

fn main() {
    let mut rl = Editor::<()>::new();

    loop {
        let line = rl.readline("user> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("{}", rep(&line));
            }
            Err(_) => break
        };
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