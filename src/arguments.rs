use std::collections::HashMap;

use crate::{
    AppState,
    money::{Money, MoneyList, YearMonth},
    print_help, print_version,
};

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
    ShowCategories,
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
            "--show-categories" | "-c" => Some(Self::ShowCategories),
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
                list(args, app_state);
            }
            Self::Exit => {
                std::process::exit(0);
            }
            Self::Version => {
                print_version();
            }
            Self::Help => {
                print_help();
            } // _ => {
              //     println!("Command not yet implemented");
              // },
        }
    }
}

fn list(args: &[Argument], app_state: &mut AppState) {
    let tracker = match &app_state.tracker {
        Some(t) => t,
        None => {
            println!("No file opened");
            return;
        }
    };
    let mut headers: Vec<&str> = vec![];
    let mut cells: HashMap<YearMonth, Vec<Money>> = HashMap::new();
    let show_categories = args.iter().any(|a| matches!(a, Argument::ShowCategories));
    let year_months = tracker.get_year_months();
    headers.push("Total");
    list_prepare_data(&tracker.total, &mut headers, &mut cells, &year_months, show_categories);
    headers.push("Incomes");
    list_prepare_data(&tracker.incomes, &mut headers, &mut cells, &year_months, show_categories);
    headers.push("Expenses");
    list_prepare_data(&tracker.expenses, &mut headers, &mut cells, &year_months, show_categories);
    let month_width: usize = 3;
    let year_width: usize = 4;
    let pad_left: usize = 2;
    let pad: usize = 2;
    let col_width: usize = 15;
    // print!("\\[\\e[1;32m\\]");
    print!("{:month_width$} {:year_width$}{:pad_left$}", "", "", "");
    for (i, header) in headers.iter().enumerate() {
        if i < headers.len() - 1 {
            print!("{:>col_width$}{:pad$}", header, "");
        } else {
            println!("{:>col_width$}", header);
        }
    }
    for year_month in year_months {
        let year = year_month.year;
        let month = year_month.month;
        print!(
            "{:month_width$} {:year_width$}{:pad_left$}",
            &month.name()[..3],
            year,
            ""
        );
        let row = &cells[&year_month];
        for (j, cell) in row.iter().enumerate() {
            if j < row.len() - 1 {
                print!("{:>#col_width$}{:pad$}", cell, "");
            } else {
                println!("{:>#col_width$}", cell);
            }
        }
    }
}

fn list_prepare_data<'a>(
    money_list: &'a MoneyList,
    headers: &mut Vec<&'a str>,
    cells: &mut HashMap<YearMonth, Vec<Money>>,
    year_months: &Vec<YearMonth>,
    show_categories: bool,
) {
    if show_categories {
        for category in money_list.categories() {
            headers.push(&category.name);
        }
    }
    for year_month in year_months {
        let row = cells.entry(*year_month).or_default();
        row.push(money_list.sum(&year_month));
        if show_categories {
            for i in 0..money_list.categories().len() {
                row.push(money_list.sum_category(&year_month, i));
            }
        }
    }
}
