use std::process::ExitCode;

#[derive(Debug)]
pub enum Command<'k> {
    Exit(ExitCode),
    Echo(&'k str),
    Type(&'k str),
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
            _ => Ok(Self::Executable(value.split_whitespace().collect())),
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
