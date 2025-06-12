pub mod command;

#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode};
use crate::command::Command;
use std::fs;

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
                    println!("{}", echo);
                }
                Command::Type(command_type) => {
                    println!("{}", handle_command_type(command_type));
                }
                Command::Pwd(pwd) => {
                    match get_absolute_path(pwd) {
                        Ok(path) => {
                            println!("{}", path.display())
                        }
                        Err(e) => {
                            println!("Error getting absolute path: {}", e);
                        }
                    }
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
        "echo" | "exit" | "type" | "pwd" => {
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
    let path = std::env::var("PATH").ok()?;
    for dir in path.split(":") {
        match std::fs::read_dir(Path::new(dir)) {
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

fn get_absolute_path(relative_path: &str) -> io::Result<PathBuf> {
    fs::canonicalize(relative_path)
}