mod add;
mod convert;
mod list;

use crate::{
    SETTINGS, SETTINGS_PATH,
    commands::{add::add, convert::convert, list::list},
    expressions::Expression,
    money::Tracker,
    serial::{self, save_file},
};

pub struct ArgList {
    command: Option<Command>,
    args: Vec<Argument>,
}

impl ArgList {
    pub fn parse(args_raw: &[String]) -> Option<Self> {
        let mut index: usize = 0;
        let mut args: Vec<Argument> = Vec::new();
        let mut command: Option<Command> = None;
        while index < args_raw.len() {
            if let Some(arg) = Argument::parse(args_raw, &mut index) {
                args.push(arg);
            } else if let Some(cmd) = Command::parse(args_raw, &mut index) {
                if command.is_some() {
                    return None;
                } else {
                    command = Some(cmd);
                }
            } else {
                return None;
            }
            index += 1;
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
    Version,
    Help,
    File(String),
    ShowCategories,
    Description(String),
}

pub(crate) enum Command {
    List,
    Add(MoneyListType, String, Expression),
    Convert(String, String),
}

#[derive(Clone, Copy)]
pub(crate) enum MoneyListType {
    Total,
    Income,
    Expense,
}

trait Parse {
    fn parse(args: &[String], index: &mut usize) -> Option<Self>
    where
        Self: Sized;
}

impl Parse for Argument {
    fn parse(args: &[String], index: &mut usize) -> Option<Self> {
        match args.get(*index)?.as_str() {
            "-v" | "--version" => Some(Self::Version),
            "-h" | "--help" => Some(Self::Help),
            "-f" | "--file" => {
                *index += 1;
                Some(Self::File(args.get(*index)?.to_owned()))
            }
            "-c" | "--show-categories" => Some(Self::ShowCategories),
            "-d" | "--description" => {
                *index += 1;
                Some(Self::Description(args.get(*index)?.to_owned()))
            }
            _ => None,
        }
    }
}

impl Parse for Command {
    fn parse(args: &[String], index: &mut usize) -> Option<Self> {
        match args.get(*index)?.as_str() {
            "list" => Some(Self::List),
            "add" => {
                *index += 1;
                let list_type = MoneyListType::parse(args, index)?;
                *index += 1;
                let cat_name = args.get(*index)?.to_owned();
                *index += 1;
                let formula = args.get(*index)?;
                Some(Self::Add(
                    list_type,
                    cat_name,
                    formula.parse().ok()?,
                ))
            }
            "convert" => {
                *index += 1;
                let input_file = args.get(*index)?.to_owned();
                *index += 1;
                let output_file = args.get(*index)?.to_owned();
                Some(Self::Convert(input_file, output_file))
            }
            _ => None,
        }
    }
}

impl Parse for MoneyListType {
    fn parse(args: &[String], index: &mut usize) -> Option<Self>
    where
        Self: Sized,
    {
        match args.get(*index)?.as_str() {
            "total" | "t" => Some(Self::Total),
            "income" | "i" => Some(Self::Income),
            "expense" | "e" => Some(Self::Expense),
            _ => None,
        }
    }
}

impl Command {
    pub fn run(&self, args: &[Argument]) -> Result<(), String> {
        match self {
            Self::List => list(args, &load_tracker(args)?),
            Self::Add(list_type, cat_name, expression) => add(
                args,
                &mut load_tracker(args)?,
                *list_type,
                cat_name,
                expression,
            )?,
            Self::Convert(input_file, output_file) => convert(input_file, output_file)?,
        }
        // save settings file
        if let Some(ref path) = *SETTINGS_PATH {
            SETTINGS
                .read()
                .map_err(|_| "Settings lock poisoned")?
                .save(path)?;
        }
        Ok(())
    }
}

fn load_tracker(args: &[Argument]) -> Result<Tracker, String> {
    let mut file_arg: Option<String> = None;
    for arg in args {
        match arg {
            Argument::File(f) => file_arg = Some(f.to_owned()),
            _ => {}
        }
    }
    let filename = &file_arg.unwrap_or(
        SETTINGS
            .read()
            .map_err(|_| "Settings lock poisoned")?
            .get("lastOpenedFile")
            .ok_or("Unable to find last opened file. Use --file to specify a file to open")?,
    );
    SETTINGS
        .write()
        .map_err(|_| "Settings lock poisoned")?
        .set("lastOpenedFile".to_string(), filename)
        .map_err(|e| e.to_string())?;
    serial::load_file(filename)
}

fn save_tracker(tracker: &Tracker) -> Result<(), String> {
    match SETTINGS
        .read()
        .map_err(|_| "Settings lock poinsoned")?
        .get::<String>("lastOpenedFile")
    {
        Some(filename) => {
            save_file(&filename, tracker).map_err(|msg| format!("Unable to save file: {msg}"))
        }
        None => Err("Unable to find path to save the file. No changes can be saved.".to_owned()),
    }
}
