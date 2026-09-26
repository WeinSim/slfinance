mod add;
mod categories;
mod convert;
mod convert_back;
mod list;
mod remove;

use chrono::{Datelike, Month, NaiveDate};

use crate::{
    CONFIG, MONTHS, SETTINGS, SETTINGS_PATH, TODAY,
    commands::{
        add::add,
        categories::{add_category, remove_category},
        convert::convert,
        convert_back::convert_back,
        list::{list, list_categories},
        remove::remove,
    },
    expressions::Expression,
    money::{Tracker, YearMonth},
    serial::{self, save_file},
};

struct ArgsIter<'a> {
    args: &'a [String],
    index: usize,
    short_iter: Option<(Vec<char>, usize)>,
}

impl<'a> ArgsIter<'a> {
    fn new(args: &'a [String]) -> Self {
        Self {
            args,
            index: 0,
            short_iter: None,
        }
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

    fn next_arg(&mut self) -> Arg<'a> {
        match &mut self.short_iter {
            Some((v, i)) => {
                let ret = v[*i];
                *i += 1;
                if *i == v.len() {
                    self.short_iter = None;
                }
                Arg::Short(ret)
            }
            None => {
                let Some(arg) = self.args.get(self.index) else {
                    return Arg::None;
                };
                self.index += 1;
                if let Some(name) = arg.strip_prefix('-') {
                    if let Some(long_name) = name.strip_prefix('-') {
                        Arg::Long(long_name)
                    } else {
                        if name.is_empty() {
                            return Arg::Invalid("");
                        }
                        let chars: Vec<char> = name.chars().collect();
                        let ret = chars[0];
                        if chars.len() > 1 {
                            self.short_iter = Some((chars, 1));
                        }
                        Arg::Short(ret)
                    }
                } else {
                    Arg::Invalid(arg)
                }
            }
        }
    }

    fn peek(&self) -> Option<&String> {
        self.args.get(self.index)
    }
}

enum Arg<'a> {
    Short(char),
    Long(&'a str),
    Invalid(&'a str),
    None,
}

#[derive(Default)]
pub struct Arguments {
    pub command: Option<Command>,
    pub version: Option<bool>,
    pub help: Option<bool>,
    pub file: Option<String>,
    pub show_categories: Option<bool>,
    pub category: Option<String>,
    pub total: bool,
    pub incomes: bool,
    pub expenses: bool,
    pub description: Option<String>,
    pub date: Option<Option<NaiveDate>>,
    pub month: Option<Month>,
    pub year: Option<i32>,
    pub wide: Option<bool>,
}

impl Arguments {
    pub fn get_year_month(&self) -> Option<YearMonth> {
        self.month.map(|month| YearMonth {
            month,
            year: self.year.unwrap_or_else(|| TODAY.year()),
        })
    }

    pub fn get_list_type(&self) -> Result<MoneyListType, String> {
        match (&self.total, &self.incomes, &self.expenses) {
            (true, false, false) => Ok(MoneyListType::Total),
            (false, true, false) => Ok(MoneyListType::Incomes),
            (false, false, true) => Ok(MoneyListType::Expenses),
            _ => Err("must specify exactly one of --total, --incomes or --expenses".to_owned()),
        }
    }

    pub fn parse(args_raw: &[String]) -> Result<Self, String> {
        let mut iter = ArgsIter::new(args_raw);
        let mut args = Self::default();
        // parse first argument as command
        if let Some(first) = iter.peek() {
            if !first.starts_with('-') {
                args.command = Some(Command::parse(&mut iter)?);
            }
        } else {
            return Err(CONFIG.help_message.to_owned());
        }
        // parse remaining arguments
        loop {
            match iter.next_arg() {
                Arg::Short('v') | Arg::Long("version") => {
                    Self::set(&mut args.version, true, "version")?
                }
                Arg::Short('h') | Arg::Long("help") => Self::set(&mut args.help, true, "help")?,
                Arg::Short('f') | Arg::Long("file") => {
                    Self::set_arg(&mut args.file, &mut iter, Self::parse_str, "file")?
                }
                Arg::Short('s') | Arg::Long("show-categories") => {
                    Self::set(&mut args.show_categories, true, "show-categories")?
                }
                Arg::Short('c') | Arg::Long("category") => {
                    Self::set_arg(&mut args.category, &mut iter, Self::parse_str, "category")?
                }
                // no duplicate checks for these three
                Arg::Short('t') | Arg::Long("total") => args.total = true,
                Arg::Short('i') | Arg::Long("incomes") => args.incomes = true,
                Arg::Short('e') | Arg::Long("expenses") => args.expenses = true,
                Arg::Short('d') | Arg::Long("description") => {
                    Self::set_arg(
                        &mut args.description,
                        &mut iter,
                        Self::parse_str,
                        "description",
                    )?;
                }
                Arg::Short('D') | Arg::Long("date") => {
                    Self::set_arg(&mut args.date, &mut iter, Self::parse_date, "date")?;
                }
                Arg::Short('m') | Arg::Long("month") => {
                    Self::set_arg(&mut args.month, &mut iter, Self::parse_month, "month")?;
                }
                Arg::Short('y') | Arg::Long("year") => {
                    Self::set_arg(&mut args.year, &mut iter, Self::parse_year, "year")?;
                }
                Arg::Short('w') | Arg::Long("wide") => Self::set(&mut args.wide, true, "wide")?,
                Arg::Short(c) => return Err(format!("unknown argument: '-{c}'")),
                Arg::Long(n) => return Err(format!("unknown argument: '--{n}'")),
                Arg::Invalid(a) => return Err(format!("invalid argument: '{a}'")),
                Arg::None => break,
            }
        }
        // if neither of --total, --incomes and --expenses were given, all three are set
        // to true
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
        Ok(match s {
            "" => None,
            "." => Some(*TODAY),
            _ => {
                if let Ok(day) = s.parse::<u32>() {
                    let ym = YearMonth::from_naive_date(*TODAY);
                    Some(
                        NaiveDate::from_ymd_opt(ym.year, ym.month.number_from_month(), day)
                            .ok_or(format!("invalid day of current month: {day}"))?,
                    )
                } else {
                    Some(s.parse::<NaiveDate>().map_err(|e| e.to_string())?)
                }
            }
        })
    }

    fn parse_month(s: &str) -> Result<Month, String> {
        // "." defaults to the current month
        if s == "." {
            return Ok(YearMonth::month_from_naive_date(*TODAY));
        }
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
        // "." defaults to the current year
        let current_year = TODAY.year();
        if s == "." {
            return Ok(current_year);
        }
        // if the input starts with a 0, we always return it as is
        let parsed = s.parse::<i32>().map_err(|e| e.to_string());
        if s.starts_with('0') {
            return parsed;
        }
        // otherwise, we adjust values between 0 - 99 to the most reasonable century
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
    Add(Expression),
    Remove(usize),
    Convert(String, String),
    ConvertBack(String, bool),
    ListCategories,
    AddCategory(String),
    RemoveCategory(String),
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
                "ls" | "list" => Ok(Self::List),
                "a" | "add" => {
                    let formula = iter.next().ok_or("expected formula")?.parse()?;
                    Ok(Self::Add(formula))
                }
                "rm" | "remove" => {
                    let index = iter
                        .next()
                        .ok_or("expected index")?
                        .parse::<usize>()
                        .map_err(|e| e.to_string())?;
                    Ok(Self::Remove(index))
                }
                "convert" => {
                    let input_file = iter.next().ok_or("expected input file")?.to_owned();
                    let output_file = iter.next().ok_or("expected output file")?.to_owned();
                    Ok(Self::Convert(input_file, output_file))
                }
                "convert-back" => {
                    let output_file = iter.next().ok_or("expected output file")?.to_owned();
                    let german = match iter.next() {
                        Some("english") => false,
                        Some("german") => true,
                        _ => {
                            return Err("Must specify either 'english' or 'german' as language:
                        slfinance convert-back <OUTPUT_FILE> english"
                                .to_owned());
                        }
                    };
                    Ok(Self::ConvertBack(output_file, german))
                }
                "lsc" | "list-categories" => Ok(Self::ListCategories),
                "ac" | "add-category" => {
                    let category = iter.next().ok_or("expected category name")?.to_owned();
                    Ok(Self::AddCategory(category))
                }
                "rmc" | "remove-category" => {
                    let category = iter.next().ok_or("expected category name")?.to_owned();
                    Ok(Self::RemoveCategory(category))
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
            Self::Add(expression) => {
                add(
                    args,
                    &mut load_tracker(args)?,
                    args.get_list_type().unwrap_or(MoneyListType::Expenses),
                    args.category.as_deref(),
                    expression,
                )?;
            }
            Self::Remove(index) => {
                let year_month = args
                    .get_year_month()
                    .unwrap_or(YearMonth::from_naive_date(*TODAY));
                remove(
                    &mut load_tracker(args)?,
                    args.get_list_type().unwrap_or(MoneyListType::Expenses),
                    year_month,
                    *index,
                )?;
            }
            Self::Convert(input_file, output_file) => convert(input_file, output_file)?,
            Self::ConvertBack(output_file, german) => convert_back(&mut load_tracker(args)?, output_file, *german)?,
            Self::ListCategories => list_categories(args, &load_tracker(args)?)?,
            Self::AddCategory(name) => {
                add_category(&mut load_tracker(args)?, name, args.get_list_type()?)?
            }
            Self::RemoveCategory(name) => {
                remove_category(&mut load_tracker(args)?, name, args.get_list_type()?)?
            }
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
