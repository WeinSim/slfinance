mod add;
mod convert;
mod list;

use std::collections::HashSet;
use std::mem::discriminant;

use chrono::{Month, NaiveDate};

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
        // TODO: may be this whole section can be improved. it looks kind of ugly at the
        // moment

        // verify that args match the given command, that no arguments are listed more
        // than once and that arguments don't conflict with each other (e.g. --date
        // and --month cannot be given together).
        let mut seen = HashSet::new();
        for arg in &args {
            // return None if there are duplicate arguments
            if !seen.insert(discriminant(arg)) {
                return None;
            }
            // return None if the argument doesn't match the specified command
            let is_valid = match arg {
                Argument::Version | Argument::Help => command.is_none(),
                Argument::File(_) => {
                    matches!(command, Some(Command::Add(_, _, _)) | Some(Command::List))
                }
                Argument::ShowCategories => matches!(command, Some(Command::List)),
                Argument::Description(_) | Argument::Date(_) => {
                    matches!(command, Some(Command::Add(_, _, _)))
                }
            };
            if !is_valid {
                return None;
            }
        }
        if seen.contains(&discriminant(&Argument::Date(None)))
            && seen.contains(&discriminant(&Argument::Month(Month::January)))
        {
            return None;
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

#[derive(PartialEq, Eq, Hash)]
pub enum Argument {
    Version,
    Help,
    File(String),
    ShowCategories,
    Description(String),
    Date(Option<NaiveDate>),
    Month(Month),
}

pub(crate) enum Command {
    List,
    Add(MoneyListType, String, Expression),
    Convert(String, String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
            "-D" | "--date" => {
                *index += 1;
                let date = args.get(*index)?;
                Some(Self::Date(if !date.is_empty() {
                    Some(date.parse::<NaiveDate>().ok()?)
                } else {
                    None
                }))
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
                Some(Self::Add(list_type, cat_name, formula.parse().ok()?))
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
            "t" | "total" => Some(Self::Total),
            "i" | "income" => Some(Self::Income),
            "e" | "expense" => Some(Self::Expense),
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
    let file_arg = args.iter().find_map(|a| match a {
        Argument::File(f) => Some(f.to_owned()),
        _ => None,
    });
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
