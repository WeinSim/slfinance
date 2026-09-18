use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt::Write;

use crate::{
    commands::Arguments,
    money::{Money, MoneyList, Tracker, YearMonth},
    sutil,
};

pub fn list(args: &Arguments, tracker: &Tracker) {
    let show_categories = args.show_categories.unwrap_or(false);
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
    // can fit at most -999,999.99€
    let col_width: usize = 13;
    let pad: usize = 2;
    table.print(month_width, year_width, pad_left, col_width, pad);
}

struct Table<'a> {
    headers: Vec<Header<'a>>,
    cells: HashMap<YearMonth, Vec<Cell>>,
    row_keys: Vec<RowKey>,
}

struct Header<'a> {
    name: &'a str,
    bold: bool,
    lines: Option<(usize, Vec<&'a str>)>,
}

struct Cell {
    money: Money,
    color: bool,
}

enum RowKey {
    YearMonth(YearMonth),
    Dots,
}

impl<'a> Header<'a> {
    fn new(name: &'a str, bold: bool) -> Self {
        Self {
            name,
            bold,
            lines: None,
        }
    }

    fn get_lines(&mut self, col_width: usize) -> &[&'a str] {
        if self
            .lines
            .as_ref()
            .is_none_or(|(width, _)| *width != col_width)
        {
            let lines = sutil::split_into_lines(self.name, col_width);
            self.lines = Some((col_width, lines));
        }
        &self.lines.as_ref().unwrap().1
    }

    fn get_num_lines(&mut self, col_width: usize) -> usize {
        self.get_lines(col_width).len()
    }

    fn print_line(&mut self, line_number: usize, col_width: usize) {
        let line = match self.get_lines(col_width).get(line_number) {
            Some(l) => l,
            None => "",
        };
        if self.bold {
            print!("{:<col_width$.col_width$}", line.bold());
        } else {
            print!("{:<col_width$.col_width$}", line);
        }
    }

    // fn print(&self, col_width: usize) {
    //     // we have to copy the name even if it is short enough because we cannot hand back
    //     // a reference to buf (since it doesn't live long enough)
    //     let num_chars = self.name.chars().count();
    //     let name_to_print = if num_chars <= col_width {
    //         self.name.to_owned()
    //     } else {
    //         let mut buf = match self.name.char_indices().nth(col_width - 3) {
    //             Some((i, _)) => self.name[0..i].to_owned(),
    //             None => self.name.to_owned(),
    //         };
    //         buf.push_str("...");
    //         buf
    //     };
    //     if self.bold {
    //         print!("{:>col_width$.col_width$}", name_to_print.bold());
    //     } else {
    //         print!("{:>col_width$.col_width$}", name_to_print);
    //     }
    // }
}

impl<'a> Table<'a> {
    fn new(year_months: &Vec<YearMonth>) -> Self {
        // pad year_months to fill in gaps
        // // doing a double reverse gives constant O(n) time complexity, instead of
        // // best-case O(1) (if no Dots have to be inserted at all) and worst-case O(n^2)
        // // (if Dots have to be inserted) at every other slot).
        // // let mut reversed = year_months
        //     .iter()
        //     .rev()
        //     .map(|ym| RowKey::YearMonth(*ym))
        //     .collect::<Vec<_>>();
        // while let Some(row_key) = reversed.pop() {
        //     row_keys.push(row_key);
        // }
        let mut row_keys: Vec<_> = year_months
            .iter()
            .map(|ym| RowKey::YearMonth(*ym))
            .collect();
        let mut i: usize = 0;
        while i < row_keys.len() - 1 {
            // the let statement and the if statement are separate because we need a
            // mutable borrow of row_keys to insert elements, which is not possible while
            // we are holding two immutable references to values inside of row_keys.
            let to_insert: Option<(YearMonth, YearMonth)> = match (&row_keys[i], &row_keys[i + 1]) {
                (RowKey::YearMonth(current), RowKey::YearMonth(next)) if next - current > 2 => {
                    Some((current.succ(), next.pred()))
                }
                _ => None,
            };
            if let Some((ym1, ym2)) = to_insert {
                row_keys.insert(i + 1, RowKey::YearMonth(ym1));
                row_keys.insert(i + 2, RowKey::Dots);
                row_keys.insert(i + 3, RowKey::YearMonth(ym2));
            }
            i += 1;
        }
        Self {
            headers: Vec::new(),
            cells: HashMap::new(),
            row_keys,
        }
    }

    fn add_column<F>(&mut self, name: &'a str, bold: bool, entries: F, color: bool)
    where
        F: Fn(&YearMonth) -> Money,
    {
        self.headers.push(Header::new(name, bold));
        for key in &self.row_keys {
            if let RowKey::YearMonth(year_month) = key {
                self.cells.entry(*year_month).or_default().push(Cell {
                    money: entries(year_month),
                    color,
                });
            }
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
        &mut self,
        month_width: usize,
        year_width: usize,
        pad_left: usize,
        col_width: usize,
        pad: usize,
    ) {
        if self.headers.is_empty() || self.row_keys.is_empty() {
            println!("[empty]");
            return;
        }
        // print headers
        let num_headers = self.headers.len();
        let num_header_rows = self
            .headers
            .iter_mut()
            .map(|h| h.get_num_lines(col_width))
            .max()
            .unwrap();
        for i in 0..num_header_rows {
            print!("{:month_width$} {:year_width$}{:pad_left$}", "", "", "");
            for (j, header) in self.headers.iter_mut().enumerate() {
                let offset = num_header_rows - header.get_num_lines(col_width);
                header.print_line(i.wrapping_sub(offset), col_width);
                if j < num_headers - 1 {
                    print!("{:pad$}", "");
                }
            }
            println!();
        }
        if num_header_rows > 1 {
            println!();
        }
        // print actual rows
        for key in &self.row_keys {
            match key {
                RowKey::YearMonth(year_month) => {
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
                RowKey::Dots => println!("..."),
            }
        }
    }
}
