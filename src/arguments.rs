use crate::{AppState, MONTHS};

pub struct ArgList {
    command: Option<Command>,
    args: Vec<Argument>,
}

impl ArgList {
    pub fn parse(args_raw: &[String]) -> Option<Self> {
        let mut arg_index: usize = 0;
        let mut args: Vec<Argument> = Vec::new();
        let mut command: Option<Command> = None;
        while arg_index < args_raw.len() {
            match Argument::parse(args_raw, &mut arg_index)? {
                Argument::Command(cmd) => {
                    if args.is_empty() && command.is_none() {
                        command = Some(cmd);
                    } else {
                        return None;
                    }
                }
                arg => {
                    args.push(arg);
                }
            }
            arg_index += 1;
        }
        Some(Self { command, args })
    }

    pub fn args(&self) -> &Vec<Argument> {
        &self.args
    }

    pub fn command(&self) -> &Option<Command> {
        &self.command
    }
}

pub enum Argument {
    Command(Command),
    Version,
    Help,
    File(String),
}

impl Argument {
    pub fn parse(args_raw: &[String], index: &mut usize) -> Option<Self> {
        match args_raw.get(*index)?.as_str() {
            "--version" | "-v" => Some(Self::Version),
            "--help" | "-h" => Some(Self::Help),
            "--file" | "-f" => {
                *index += 1;
                Some(Self::File(args_raw.get(*index)?.to_owned()))
            }
            command => Some(Self::Command(Command::parse(command)?)),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Command {
    List,
    Help,
    Version,
    Exit,
}

impl Command {
    fn parse(command: &str) -> Option<Self> {
        match command {
            "list" => Some(Self::List),
            "help" => Some(Self::Help),
            "version" => Some(Self::Version),
            "exit" => Some(Self::Exit),
            _ => None,
        }
    }

    pub fn run(&self, args: &[Argument], app_state: &mut AppState) {
        match self {
            Self::List => {
                let tracker = match &app_state.tracker {
                    Some(t) => t,
                    None => {
                        println!("No file opened");
                        return;
                    }
                };
                let year_width: usize = 8;
                let pad_left: usize = 2;
                let pad: usize = 2;
                let col_width: usize = 10;
                println!(
                    "{:year_width$}{:pad_left$}{:>col_width$}{:>pad$}{:>col_width$}{:>pad$}{:>col_width$}",
                    "", "", "total", "", "income", "", "expenses"
                );
                for year in tracker.get_years() {
                    for month in MONTHS {
                        print!("{:3} {year:4}  ", &month.name()[..3]);
                        for total in &tracker.total {
                            if total.year == year && &total.month == month {
                                print!("{:>col_width$}", total.amount);
                            }
                        }
                        println!();
                    }
                }
            }
            _ => {
                println!("Command not yet implemented");
            }
        }
    }
}
