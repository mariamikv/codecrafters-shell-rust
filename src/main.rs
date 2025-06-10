#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        if starts_with_echo(&input) {
            println!("{}", format_input(input.trim()));
        } else if "exit 0" == input.trim() {
            break;
        } else if starts_with_type(&input) {
            println!("{}", handle_command_type(input.trim()));
        } else {
            println!("{}: command not found", input.trim());
        }
    }
}

fn starts_with_echo(s: &str) -> bool {
    s.starts_with("echo ")
}

fn format_input(s: &str) -> String {
    s.replace("echo ", "")
}

fn starts_with_type(s: &str) -> bool {
    s.starts_with("type ")
}

fn handle_command_type(s: &str) -> String {
    let command = s.split_whitespace().nth(1);
    match command {
        Some("echo") | Some("exit") | Some("type") => {
            format!("{} is a shell builtin", command.unwrap())
        },
        _ => {
            format!("{}: not found", command.unwrap())
        }
    }
}