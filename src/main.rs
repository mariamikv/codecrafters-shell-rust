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

        let (command_part, redirect_stdout, redirect_stderr) = split_redirect_input(input);

        match Command::handle_command(&command_part) {
            Ok(command) => match command {
                Command::Exit(code) => return code,
                Command::Echo(echo) => {
                    let output = parse_shell_arguments(echo).join(" ");

                    if let Some(out) = redirect_stdout.as_ref() {
                        if let Some(parent) = Path::new(out).parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        if let Ok(mut file) = File::create(out) {
                            let _ = writeln!(file, "{}", output);
                        }
                    } else {
                        println!("{}", output);
                    }

                    if let Some(err) = redirect_stderr.as_ref() {
                        if let Some(parent) = Path::new(err).parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let _ = File::create(err);
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
                            if let Some(err_path) = &redirect_stderr {
                                if let Ok(mut file) = File::create(err_path) {
                                    let _ = writeln!(file, "cd: {}: No such file or directory", path.display());
                                }
                            } else {
                                eprintln!("cd: {}: No such file or directory", path.display());
                            }
                        }
                    }
                }
                Command::Cat(content) => {
                    let mut stderr_file = redirect_stderr
                        .as_ref()
                        .and_then(|path| File::create(path).ok());

                    let output = handle_cat_content(content, stderr_file.as_mut());
                    create_file_path(redirect_stdout, output);
                }
                Command::Executable(executable) => {
                    let mut cmd = std::process::Command::new(executable[0]);
                    cmd.args(&executable[1..]);

                    if let Some(ref path) = redirect_stdout {
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

                    if let Some(ref path) = redirect_stderr {
                        match File::create(path) {
                            Ok(file) => {
                                cmd.stderr(Stdio::from(file));
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
                                if redirect_stdout.is_none() {
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
    let path_env = env::var("PATH").ok()?;
    for dir in path_env.split(':') {
        let full_path = Path::new(dir).join(command);
        if full_path.exists() && is_executable(&full_path) {
            return Some(full_path);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn handle_cat_content(content: &str, mut redirect_stderr: Option<&mut File>) -> String {
    let args = parse_shell_arguments(content);
    let mut output = String::new();

    for file in args {
        match fs::read_to_string(&file) {
            Ok(content) => {
                output.push_str(content.trim_end());
            }
            Err(err) => {
                let msg = if err.kind() == io::ErrorKind::NotFound {
                    format!("cat: {}: No such file or directory", file)
                } else {
                    format!("cat: {}: {}", file, err)
                };

                match redirect_stderr.as_mut() {
                    Some(f) => {
                        let _ = writeln!(f, "{}", msg);
                    }
                    None => {
                        eprintln!("{}", msg);
                    }
                }
            }
        }
    }

    output.trim_end().to_string()
}

fn create_file_path(
    redirect_target: Option<String>,
    output: String,
) {
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