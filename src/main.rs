pub mod command;

#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode};
use crate::command::Command;
use std::{env, fs};

fn main() -> ExitCode {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            continue;
        }
        let input = input.trim_end();
        if input.is_empty() {
            continue;
        }

        match Command::handle_command(input) {
            Ok(command) => match command {
                Command::Exit(code) => return code,
                Command::Echo(echo) => {
                    println!("{}", parse_shell_arguments(echo).join(" "));
                }
                Command::Type(command_type) => {
                    println!("{}", handle_command_type(command_type));
                }
                Command::Pwd(_) => {
                    match env::current_dir() {
                        Ok(path) => println!("{}", path.display()),
                        Err(e) => println!("Error getting current dir: {}", e),
                    }
                }
                Command::Cd(path_str) => {
                    let fixed_path = path_str.trim();
                    let resolved_path = if fixed_path == "~" {
                        env::var("HOME").unwrap_or_else(|_| String::from("/"))
                    } else {
                        fixed_path.to_string()
                    };

                    let path = Path::new(&resolved_path);

                    match env::set_current_dir(path) {
                        Ok(_) => {}
                        Err(_) => {
                            eprintln!("cd: {}: No such file or directory", path.display());
                            io::stderr().flush().unwrap();
                        }
                    }
                }
                Command::Cat(content) => {
                    println!("{}", handle_cat_content(content));
                }
                Command::Executable(executable) => {
                    match std::process::Command::new(executable[0]).args(&executable[1..]).spawn() {
                        Ok(program) => match program.wait_with_output() {
                            Ok(output) => {
                                print!("{}", String::from_utf8_lossy(&output.stderr));
                                print!("{}", String::from_utf8_lossy(&output.stdout));
                            }
                            Err(err) => {
                                println!("{}", err.to_string());
                            },
                        },
                        Err(_) => {
                            println!("{}: command not found", executable[0]);
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}

fn handle_command_type(command: &str) -> String {
    match command {
        "echo" | "exit" | "type" | "pwd" | "cd" => {
            format!("{command} is a shell builtin")
        },
        _ => match handle_path(command) {
            Some(path) => {
                format!("{command} is {}", path.display())
            }
            None => {
                format!("{command}: not found")
            }
        }
    }
}

fn handle_path(command: &str) -> Option<PathBuf> {
    let path = env::var("PATH").ok()?;
    for dir in path.split(":") {
        match fs::read_dir(Path::new(dir)) {
            Ok(mut read_dir) => {
                if let Some(path) = read_dir.find(|maybe_dir_entry| {
                    maybe_dir_entry
                        .as_ref()
                        .is_ok_and(|dir_entry| dir_entry.file_name() == command)
                }) {
                    if let Ok(path) = path {
                        return Some(path.path());
                    }
                }
            }
            Err(_) => continue,
        }
    }
    None
}

fn parse_shell_arguments(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_quotes = !in_quotes;
                if !in_quotes && current.is_empty() {
                    continue;
                }
            }
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
                while let Some(' ') = chars.peek() {
                    chars.next();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

fn handle_cat_content(content: &str) -> String {
    let args = parse_shell_arguments(content);

    let mut output = String::new();
    for path in args {
        match fs::read_to_string(&path) {
            Ok(content) => output.push_str(&content),
            Err(err) => eprintln!("Error reading {}: {}", path, err),
        }
    }

   output.trim_end().to_string()
}