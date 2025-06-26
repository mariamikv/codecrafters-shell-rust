use std::process::ExitCode;

#[derive(Debug)]
pub enum Command<'k> {
    Exit(ExitCode),
    Echo(&'k str),
    Type(&'k str),
    Pwd(&'k str),
    Cd(&'k str),
    Cat(&'k str),
    Executable(Vec<&'k str>),
}

impl<'k> Command<'k> {
    pub fn handle_command(value: &'k str) -> Result<Self, anyhow::Error> {
        let mut commands = value.split_whitespace();
        match commands.next().unwrap_or("") {
            "exit" => {
               Ok(Self::Exit(parse_exit_command(value)?))
            },
            "echo" => {
                Ok(Self::Echo(value.strip_prefix("echo ").unwrap_or("")))
            },
            "type" =>  match commands.next() {
                Some(s) => {
                    Ok(Self::Type(s))
                }
                None => anyhow::bail!("type requires one argument"),
            },
            "pwd" => {
                Ok(Self::Pwd("./your_program.sh"))
            }
            "cd" => {
                let arg = value.get(2..).unwrap_or("").trim();
                Ok(Self::Cd(arg))
            }
            "cat" => {
                Ok(Self::Cat(value.strip_prefix("cat ").unwrap_or("")))
            }
            _ => {
                let parsed = parse_shell_arguments(value);
                let mut slices = Vec::new();
                let mut start_idx = 0;

                for word in parsed {
                    if let Some(found) = value[start_idx..].find(&word) {
                        let begin = start_idx + found;
                        let end = begin + word.len();
                        slices.push(&value[begin..end]);
                        start_idx = end;
                    } else {
                        // fallback if exact match isn't found (edge case)
                        slices.push(Box::leak(word.into_boxed_str()));
                    }
                }

                Ok(Command::Executable(slices))
            }
        }
    }
}

fn parse_exit_command(command: &str) -> Result<ExitCode, anyhow::Error> {
    let exit_code = command
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse::<u8>()?;

    Ok(ExitCode::from(exit_code))
}

pub fn parse_shell_arguments(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote_char: Option<char> = None;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if let Some('"') = quote_char {
            if escape_next {
                match c {
                    '"' | '\\' => current.push(c),
                    _ => {
                        current.push('\\');
                        current.push(c);
                    }
                }
                escape_next = false;
                continue;
            }

            match c {
                '\\' => escape_next = true,
                '"' => quote_char = None,
                _ => current.push(c),
            }
        } else if let Some('\'') = quote_char {
            if c == '\'' {
                quote_char = None;
            } else {
                current.push(c);
            }
        } else {
            if escape_next {
                current.push(c);
                escape_next = false;
                continue;
            }

            match c {
                '\\' => escape_next = true,
                '\'' | '"' => quote_char = Some(c),
                ' ' => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                    while let Some(' ') = chars.peek() {
                        chars.next();
                    }
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

pub fn split_redirect_input(input: &str) -> (String, Option<String>) {
    if let Some((left, right)) = input.split_once("1>") {
        (left.trim().to_string(), Some(right.trim().to_string()))
    } else if let Some((left, right)) = input.split_once('>') {
        (left.trim().to_string(), Some(right.trim().to_string()))
    } else {
        (input.trim().to_string(), None)
    }
}
