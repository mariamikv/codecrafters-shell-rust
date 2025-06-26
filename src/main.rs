pub mod command;

#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use crate::command::{split_redirect_input, Command};
use crate::command::parse_shell_arguments;
use std::{env, fs};
use std::fs::File;

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

        let (command_part, redirect_target) = split_redirect_input(input);

        match Command::handle_command(&command_part) {
            Ok(command) => match command {
                Command::Exit(code) => return code,
                Command::Echo(echo) => {
                    let output = parse_shell_arguments(echo).join(" ");

                    if let Some(ref path) = redirect_target {
                        if let Some(parent) = Path::new(path).parent() {
                            let _ = fs::create_dir_all(parent);
                        }

                        match File::create(path) {
                            Ok(mut file) => {
                                let _ = writeln!(file, "{}", output);
                            }
                            Err(e) => {
                                eprintln!("Redirection failed: {}", e);
                            }
                        }
                    } else {
                        println!("{}", output);
                    }
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
                    let mut cmd = std::process::Command::new(executable[0]);
                    cmd.args(&executable[1..]);

                    if let Some(ref path) = redirect_target {
                        match File::create(path) {
                            Ok(file) => {
                                cmd.stdout(Stdio::from(file));
                            }
                            Err(e) => {
                                eprintln!("Redirection failed: {}", e);
                                continue;
                            }
                        }
                    }

                    match cmd.spawn() {
                        Ok(child) => match child.wait_with_output() {
                            Ok(output) => {
                                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                                if redirect_target.is_none() {
                                    print!("{}", String::from_utf8_lossy(&output.stdout));
                                }
                            }
                            Err(err) => {
                                eprintln!("Execution failed: {}", err);
                            }
                        },
                        Err(_) => {
                            eprintln!("{}: command not found", executable[0]);
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

fn handle_cat_content(content: &str) -> String {
    let args = parse_shell_arguments(content);
    let mut output = String::new();

    for file in args {
        match fs::read_to_string(&file) {
            Ok(content) => {
                output.push_str(content.trim_end());
            }
            Err(err) => eprintln!("cat: {}: {}", file, err),
        }
    }

   output.trim_end().to_string()
}