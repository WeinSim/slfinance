use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt::Write;

use crate::money::{Money, MoneyList, Tracker, YearMonth};

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
}

impl Command {
    fn parse(command: &str) -> Option<Self> {
        match command {
            "list" => Some(Self::List),
            _ => None,
        }
    }

    pub fn run(&self, args: &[Argument], tracker: &mut Tracker) -> Result<(), String> {
        match self {
            Self::List => {
                list(args, tracker);
            } // _ => {
              //     println!("Command not yet implemented");
              // },
        }
        Ok(())
    }
}

fn list(args: &[Argument], tracker: &mut Tracker) {
    let show_categories = args.iter().any(|a| matches!(a, Argument::ShowCategories));
    let year_months = tracker.get_year_months();
    let mut table = Table::new(&year_months);
    table.insert_money_list(&tracker.total, "Total", show_categories, true);
    // table.add_separator();
    table.add_column("Change", true, |ym| tracker.get_total_change(ym), true);
    table.add_column("Diff", true, |ym| tracker.get_diff_total_change(ym), true);
    table.add_column(
        "Expected",
        true,
        |ym| tracker.get_expected_total_change(ym),
        true,
    );
    table.insert_money_list(&tracker.incomes, "Incomes", show_categories, false);
    table.insert_money_list(&tracker.expenses, "Expenses", show_categories, false);
    let month_width: usize = 3;
    let year_width: usize = 4;
    let pad_left: usize = 2;
    let col_width: usize = 12;
    let pad: usize = 2;
    table.print(month_width, year_width, pad_left, col_width, pad);
}

struct Table<'a> {
    headers: Vec<Header<'a>>,
    cells: HashMap<YearMonth, Vec<Cell>>,
    year_months: &'a Vec<YearMonth>,
}

struct Header<'a> {
    name: &'a str,
    bold: bool,
}

struct Cell {
    money: Money,
    color: bool,
}

impl Header<'_> {
    fn print(&self, col_width: usize) {
        // we have to copy the name even if it is short enough because we cannot hand back
        // a reference to buf (since it doesn't live long enough)
        let name_to_print = if self.name.len() <= col_width {
            self.name.to_owned()
        } else {
            let mut buf = self.name[0..col_width - 3].to_owned();
            buf.push_str("...");
            buf
        };
        if self.bold {
            print!("{:>col_width$.col_width$}", name_to_print.bold());
        } else {
            print!("{:>col_width$.col_width$}", name_to_print);
        }
    }
}

impl<'a> Table<'a> {
    fn new(year_months: &'a Vec<YearMonth>) -> Self {
        Self {
            headers: Vec::new(),
            cells: HashMap::new(),
            year_months,
        }
    }

    fn add_column<F>(&mut self, name: &'a str, bold: bool, entries: F, color: bool)
    where
        F: Fn(&YearMonth) -> Money,
    {
        self.headers.push(Header { name, bold });
        for year_month in self.year_months {
            self.cells.entry(*year_month).or_default().push(Cell {
                money: entries(year_month),
                color,
            });
        }
    }

    fn insert_money_list(
        &mut self,
        money_list: &'a MoneyList,
        name: &'a str,
        show_categories: bool,
        sum_last: bool,
    ) {
        // headers
        if !sum_last {
            self.add_column(name, true, |ym| money_list.sum(ym), false);
        }
        if show_categories {
            for (i, category) in money_list.categories().iter().enumerate() {
                self.add_column(
                    &category.name,
                    false,
                    |ym| money_list.sum_category(ym, i),
                    false,
                );
            }
        }
        if sum_last {
            self.add_column(name, true, |ym| money_list.sum(ym), false);
        }
    }

    fn print(
        &self,
        month_width: usize,
        year_width: usize,
        pad_left: usize,
        col_width: usize,
        pad: usize,
    ) {
        print!("{:month_width$} {:year_width$}{:pad_left$}", "", "", "");
        for (i, header) in self.headers.iter().enumerate() {
            header.print(col_width);
            if i < self.headers.len() - 1 {
                print!("{:pad$}", "");
            }
        }
        println!();
        for year_month in self.year_months {
            let year = year_month.year;
            let month = year_month.month;
            print!(
                "{:month_width$} {:year_width$}{:pad_left$}",
                &month.name()[..3],
                year,
                ""
            );
            let Some(row) = self.cells.get(&year_month) else {
                continue;
            };
            for (j, cell) in row.iter().enumerate() {
                let mut fmt = String::with_capacity(col_width);
                match write!(&mut fmt, "{:>#col_width$.col_width$}", cell.money) {
                    Ok(_) => {}
                    Err(e) => println!("{}", e.to_string()),
                }
                match cell.money {
                    m if cell.color && m.is_positive() => print!("{}", fmt.green()),
                    m if cell.color && m.is_negative() => print!("{}", fmt.bright_red()),
                    _ => print!("{}", fmt),
                }
                if j < row.len() - 1 {
                    print!("{:pad$}", "");
                }
            }
            println!();
        }
    }
}
