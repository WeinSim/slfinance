mod add;
mod convert;
mod list;

use chrono::{Month, NaiveDate};

use crate::{
    CONFIG, SETTINGS, SETTINGS_PATH,
    commands::{add::add, convert::convert, list::list},
    expressions::Expression,
    money::Tracker,
    serial::{self, save_file},
};

struct ArgsIter<'a> {
    args: &'a [String],
    index: usize,
}

impl<'a> ArgsIter<'a> {
    fn new(args: &'a [String]) -> Self {
        Self { args, index: 0 }
    }

    fn next(&mut self) -> Option<&str> {
        if self.index < self.args.len() {
            let ret = &self.args[self.index];
            self.index += 1;
            Some(ret)
        } else {
            None
        }
    }

    fn go_back(&mut self) {
        self.index -= 1;
    }
}

#[derive(Default)]
pub struct Arguments {
    pub command: Option<Command>,
    pub version: Option<bool>,
    pub help: Option<bool>,
    pub file: Option<String>,
    pub show_categories: Option<bool>,
    pub description: Option<String>,
    pub date: Option<Option<NaiveDate>>,
    pub month: Option<Month>,
}

impl Arguments {
    pub fn parse(args_raw: &[String]) -> Result<Self, String> {
        let mut iter = ArgsIter::new(args_raw);
        let mut args = Self::default();
        if let Some(first) = iter.next() {
            if !first.starts_with('-') {
                iter.go_back();
                args.command = Some(Command::parse(&mut iter)?);
            }
        } else {
            return Err(CONFIG.help_message.clone());
        }
        while let Some(arg) = iter.next() {
            match arg {
                "-v" | "--version" => Self::set(&mut args.version, true, "version")?,
                "-h" | "--help" => Self::set(&mut args.help, true, "help")?,
                "-f" | "--file" => {
                    Self::set_arg(&mut args.file, &mut iter, Self::parse_str, "file")?
                }
                "-c" | "--show-categories" => {
                    Self::set(&mut args.show_categories, true, "show-categories")?
                }
                "-d" | "--description" => {
                    Self::set_arg(
                        &mut args.description,
                        &mut iter,
                        Self::parse_str,
                        "description",
                    )?;
                }
                "-D" | "--date" => {
                    Self::set_arg(
                        &mut args.date,
                        &mut iter,
                        |s| {
                            if s.is_empty() {
                                Ok::<_, String>(None)
                            } else {
                                Ok(Some(s.parse::<NaiveDate>().map_err(|e| e.to_string())?))
                            }
                        },
                        "date",
                    )?;
                }
                a => return Err(format!("Unknown argument: '{a}'")),
            }
        }
        if args.date.is_some() && args.month.is_some() {
            return Err("Conflicting arguments '--month' and '--date'".to_owned());
        }
        Ok(args)
    }

    fn set<T>(field: &mut Option<T>, value: T, arg_name: &str) -> Result<(), String> {
        match field {
            Some(_) => Err(format!("Duplicate argument '--{arg_name}'")),
            None => {
                *field = Some(value);
                Ok(())
            }
        }
    }

    fn set_arg<T, E>(
        field: &mut Option<T>,
        iter: &mut ArgsIter,
        parse: fn(&str) -> Result<T, E>,
        arg_name: &str,
    ) -> Result<(), String>
    where
        E: ToString,
    {
        Self::set(
            field,
            parse(
                iter.next()
                    .ok_or(format!("Missing value for '{arg_name}'"))?,
            )
            .map_err(|e| e.to_string())?,
            arg_name,
        )
    }

    fn parse_str(s: &str) -> Result<String, String> {
        Ok(s.to_owned())
    }

    // fn check_arg<T>(arg: &Option<T>, is_valid: fn(Option<Command>) -> bool, M)
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
    fn parse(iter: &mut ArgsIter) -> Result<Self, String>
    where
        Self: Sized;
}

impl Command {
    fn parse(iter: &mut ArgsIter) -> Result<Self, String> {
        if let Some(command) = iter.next() {
            match command {
                "list" => Ok(Self::List),
                "add" => {
                    let list_type = MoneyListType::parse(iter)?;
                    let cat_name = iter.next().ok_or("Expected category name")?.to_owned();
                    let formula = iter.next().ok_or("Expected formula")?.parse()?;
                    Ok(Self::Add(list_type, cat_name, formula))
                }
                "convert" => {
                    let input_file = iter.next().ok_or("Expected input file")?.to_owned();
                    let output_file = iter.next().ok_or("Expected output file")?.to_owned();
                    Ok(Self::Convert(input_file, output_file))
                }
                _ => Err(format!("Invalid command: '{command}'")),
            }
        } else {
            Err("Expected command".to_owned())
        }
    }
}

impl Parse for MoneyListType {
    fn parse(iter: &mut ArgsIter) -> Result<Self, String>
    where
        Self: Sized,
    {
        if let Some(s) = iter.next() {
            match s {
                "t" | "total" => Ok(Self::Total),
                "i" | "income" => Ok(Self::Income),
                "e" | "expense" => Ok(Self::Expense),
                _ => Err(format!("Invalid money list type: '{s}'")),
            }
        } else {
            Err("Expected money list type".to_owned())
        }
    }
}

impl Command {
    pub fn run(&self, args: &Arguments) -> Result<(), String> {
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

fn load_tracker(args: &Arguments) -> Result<Tracker, String> {
    let filename = &args.file.clone().unwrap_or(
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
