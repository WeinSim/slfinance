mod convert;
mod list;

use chrono::{Datelike, Local, Month};

use crate::{
    SETTINGS, SETTINGS_PATH,
    commands::{convert::convert, list::list},
    money::{Money, MoneyChange, Tracker, YearMonth},
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
}

pub(crate) enum Command {
    List,
    Add(MoneyListType, String, Money),
    Convert(String, String),
}

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
            "--version" | "-v" => Some(Self::Version),
            "--help" | "-h" => Some(Self::Help),
            "--file" | "-f" => {
                *index += 1;
                Some(Self::File(args.get(*index)?.to_owned()))
            }
            "--show-categories" | "-c" => Some(Self::ShowCategories),
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
                let euros = args.get(*index)?.parse::<i64>().ok()?;
                Some(Self::Add(list_type, cat_name, Money { cents: euros * 100 }))
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
            Self::Add(list_type, cat_name, amount) => {
                let mut tracker = load_tracker(args)?;
                let date = Local::now().date_naive();
                let list = match list_type {
                    MoneyListType::Total => &mut tracker.total,
                    MoneyListType::Income => &mut tracker.incomes,
                    MoneyListType::Expense => &mut tracker.expenses,
                };
                let cat_id = list.find_or_create_category(cat_name);
                list.add_entry(
                    YearMonth {
                        year: date.year(),
                        month: Month::try_from(date.month() as u8).unwrap(),
                    },
                    MoneyChange {
                        amount: *amount,
                        date: match list_type {
                            MoneyListType::Total => None,
                            _ => Some(date),
                        },
                        category_id: Some(cat_id),
                    },
                )?;
                save_tracker(&tracker)?;
            }
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
