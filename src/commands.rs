mod add;
mod convert;
mod list;

use chrono::{Datelike, Local, Month, NaiveDate};

use crate::{
    CONFIG, MONTHS, SETTINGS, SETTINGS_PATH,
    commands::{add::add, convert::convert, list::list},
    expressions::Expression,
    money::{Tracker, YearMonth},
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

    fn peek(&self) -> Option<&String> {
        self.args.get(self.index)
    }
}

#[derive(Default)]
pub struct Arguments {
    pub command: Option<Command>,
    pub version: Option<bool>,
    pub help: Option<bool>,
    pub file: Option<String>,
    pub show_categories: Option<bool>,
    pub total: bool,
    pub incomes: bool,
    pub expenses: bool,
    pub description: Option<String>,
    pub date: Option<Option<NaiveDate>>,
    pub month: Option<Month>,
    pub year: Option<i32>,
}

impl Arguments {
    pub fn get_year_month(&self) -> Option<YearMonth> {
        let today = Local::now().date_naive();
        if self.month.is_none() && self.year.is_none() {
            None
        } else {
            Some(YearMonth {
                year: self.year.unwrap_or(today.year()),
                month: self
                    .month
                    .unwrap_or(YearMonth::month_from_naive_date(today)),
            })
        }
    }

    pub fn parse(args_raw: &[String]) -> Result<Self, String> {
        let mut iter = ArgsIter::new(args_raw);
        let mut args = Self::default();
        if let Some(first) = iter.peek() {
            if !first.starts_with('-') {
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
                // no duplicate checks for these three
                "-t" | "--total" => args.total = true,
                "-i" | "--incomes" => args.incomes = true,
                "-e" | "--expenses" => args.expenses = true,
                "-d" | "--description" => {
                    Self::set_arg(
                        &mut args.description,
                        &mut iter,
                        Self::parse_str,
                        "description",
                    )?;
                }
                "-D" | "--date" => {
                    Self::set_arg(&mut args.date, &mut iter, Self::parse_date, "date")?;
                }
                "-m" | "--month" => {
                    Self::set_arg(&mut args.month, &mut iter, Self::parse_month, "month")?;
                }
                "-y" | "--year" => {
                    Self::set_arg(&mut args.year, &mut iter, Self::parse_year, "year")?;
                }
                "-my" | "--month-year" => {
                    Self::set_arg(&mut args.month, &mut iter, Self::parse_month, "month")?;
                    Self::set_arg(&mut args.year, &mut iter, Self::parse_year, "year")?;
                }
                a => return Err(format!("unknown argument: '{a}'")),
            }
        }
        if !(args.total || args.incomes || args.expenses) {
            args.total = true;
            args.incomes = true;
            args.expenses = true;
        }
        if args.date.is_some() {
            if args.month.is_some() {
                return Err("conflicting arguments '--date' and '--month'".to_owned());
            }
            if args.year.is_some() {
                return Err("conflicting arguments '--date' and '--year'".to_owned());
            }
        }
        Ok(args)
    }

    fn set<T>(field: &mut Option<T>, value: T, arg_name: &str) -> Result<(), String> {
        match field {
            Some(_) => Err(format!("duplicate argument '--{arg_name}'")),
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
                    .ok_or(format!("missing value for '{arg_name}'"))?,
            )
            .map_err(|e| format!("unable to parse arg '{}': {}", arg_name, e.to_string()))?,
            arg_name,
        )
    }

    fn parse_str(s: &str) -> Result<String, String> {
        Ok(s.to_owned())
    }

    fn parse_date(s: &str) -> Result<Option<NaiveDate>, String> {
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s.parse::<NaiveDate>().map_err(|e| e.to_string())?))
        }
    }

    fn parse_month(s: &str) -> Result<Month, String> {
        // first check if input is a valid month number (1 - 12)
        if let Ok(i) = s.parse::<u8>()
            && let Ok(month) = Month::try_from(i)
        {
            return Ok(month);
        }
        // then check if input is a unique prefix of a month (case-insensitive)
        let s_lower = s.to_ascii_lowercase();
        let matches: Vec<&Month> = MONTHS
            .iter()
            .filter(|m| m.name().to_ascii_lowercase().starts_with(&s_lower))
            .collect();
        match matches.len() {
            1 => Ok(*matches[0]),
            0 => Err(format!("invalid month: '{s}'")),
            _ => Err(format!("month is not unique: '{s}'")),
        }
    }

    fn parse_year(s: &str) -> Result<i32, String> {
        let parsed = s.parse::<i32>().map_err(|e| e.to_string());
        if s.starts_with('0') {
            return parsed;
        }
        let current_year = Local::now().date_naive().year();
        let current_century = (current_year / 100) * 100;
        Ok(match parsed? {
            y if y <= current_year % 100 => y + current_century,
            y if y < 100 => y + current_century - 100,
            y => y,
        })
    }
}

pub(crate) enum Command {
    List,
    Add(String, Expression),
    Convert(String, String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoneyListType {
    Total,
    Incomes,
    Expenses,
}

trait Parse {
    fn parse(iter: &mut ArgsIter) -> Result<Self, String>
    where
        Self: Sized;
}

impl Parse for Command {
    fn parse(iter: &mut ArgsIter) -> Result<Self, String> {
        if let Some(command) = iter.next() {
            match command {
                "list" => Ok(Self::List),
                "add" => {
                    let cat_name = iter.next().ok_or("Expected category name")?.to_owned();
                    let formula = iter.next().ok_or("Expected formula")?.parse()?;
                    Ok(Self::Add(cat_name, formula))
                }
                "convert" => {
                    let input_file = iter.next().ok_or("Expected input file")?.to_owned();
                    let output_file = iter.next().ok_or("Expected output file")?.to_owned();
                    Ok(Self::Convert(input_file, output_file))
                }
                _ => Err(format!("invalid command: '{command}'")),
            }
        } else {
            Err("expected command".to_owned())
        }
    }
}

impl Command {
    pub fn run(&self, args: &Arguments) -> Result<(), String> {
        match self {
            Self::List => list(args, &load_tracker(args)?),
            Self::Add(cat_name, expression) => {
                let list_type = match (&args.total, &args.incomes, &args.expenses) {
                    (true, false, false) => MoneyListType::Total,
                    (false, true, false) => MoneyListType::Incomes,
                    (false, false, true) => MoneyListType::Expenses,
                    _ => {
                        return Err(
                            "must specify exactly one of --total, --incomes or --expenses"
                                .to_owned(),
                        );
                    }
                };
                add(
                    args,
                    &mut load_tracker(args)?,
                    list_type,
                    cat_name,
                    expression,
                )?;
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
        None => Err("unable to find path to save the file. no changes can be saved.".to_owned()),
    }
}
