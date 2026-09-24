use owo_colors::OwoColorize;
use std::fmt::{Display, Write};
use std::{collections::HashMap, hash::Hash};

use crate::money::MoneyChange;
use crate::{
    commands::Arguments,
    money::{Money, MoneyList, Tracker, YearMonth},
    sutil,
};

const MONEY_COL_WIDTH: usize = 13;
const DATE_COL_WIDTH: usize = 10;
const CATEGORY_COL_WIDTH: usize = 25;
const DESCRIPTION_COL_WIDTH: usize = 30;
const TABLE_PADDING: usize = 2;

pub fn list(args: &Arguments, tracker: &Tracker) {
    match args.get_year_month() {
        Some(ym) => list_detailed(args, tracker, ym),
        None => list_all(args, tracker),
    }
}

fn list_all(args: &Arguments, tracker: &Tracker) {
    let show_categories = args.show_categories.unwrap_or(false);
    let year_months = if let Some(year) = args.year {
        tracker
            .get_year_months()
            .iter()
            .filter(|ym| ym.year == year)
            .map(|ym| ym.to_owned())
            .collect()
    } else {
        tracker.get_year_months()
    };
    let mut table = Table::new(RowKey::from_year_months(&year_months));
    // can fit at most -999,999.99€
    table.key_width = 8;
    table.key_prec = 3;
    if args.total {
        table.add_money_list(&tracker.total, "Total", show_categories, true);
        table.add_money_column("Change", true, |ym| tracker.get_total_change(ym), true);
    }
    if args.incomes && args.expenses {
        if args.total {
            table.add_money_column("Diff", true, |ym| tracker.get_diff_total_change(ym), true);
        }
        table.add_money_column(
            "Expected",
            true,
            |ym| tracker.get_expected_total_change(ym),
            true,
        );
    }
    if args.incomes {
        table.add_money_list(&tracker.incomes, "Incomes", show_categories, false);
    }
    if args.expenses {
        table.add_money_list(&tracker.expenses, "Expenses", show_categories, false);
    }
    table.print();
}

fn list_detailed(args: &Arguments, tracker: &Tracker, year_month: YearMonth) {
    // prepare money lists
    let lists: Vec<(&MoneyList, &Vec<MoneyChange>, &str)> = get_specified_lists(args, tracker)
        .iter()
        .map(|(l, n)| (l, l.entries().get(&year_month), n))
        .filter_map(|(l, o, n)| o.as_ref().map(|v| (*l, *v, *n)))
        .collect();
    // create table
    let mut table = Table::with_num_rows(
        lists
            .iter()
            .map(|(_, v, _)| v.len())
            .max()
            .unwrap_or_default(),
    );
    table.key_width = 3;
    table.key_prec = 3;
    let mut money_indices = Vec::<usize>::new();
    for (list, vec, name) in &lists {
        money_indices.push(table.add_money_list(list, vec, name, args.wide.is_some()));
    }
    // print year and month
    println!("{} {}", year_month.month.name(), year_month.year);
    // print table
    table.print();
    if table.is_empty() {
        return;
    }
    // print sums
    let sums: Vec<Money> = lists
        .iter()
        .map(|(_, v, _)| v.iter().map(|mc| mc.amount.eval()).sum())
        .collect();
    let max_index = money_indices.iter().max().unwrap();
    let mut row1 = vec![Cell::Empty; *max_index + 1];
    let mut row2 = vec![Cell::Empty; *max_index + 1];
    for (i, c) in money_indices.iter().enumerate() {
        row1[*c] = Cell::Text {
            text: "Sum".to_owned(),
            bold: true,
        };
        row2[*c] = Cell::Money {
            money: sums[i],
            color: false,
        };
    }
    table.print_row(&row1, None);
    table.print_row(&row2, None);
}

pub fn list_categories(args: &Arguments, tracker: &Tracker) -> Result<(), String> {
    let lists = get_specified_lists(args, tracker);
    let max_num_categories = lists
        .iter()
        .map(|(l, _)| l.categories().len())
        .max()
        .unwrap();
    let mut table = Table::with_num_rows(max_num_categories);
    for (list, name) in lists {
        let width = CATEGORY_COL_WIDTH * if args.wide.is_some() { 2 } else { 1 };
        table.add_text_column(name, true, width, |i| {
            list.categories().get(*i).map(|c| "  ".to_owned() + &c.name)
        });
    }
    table.print();
    Ok(())
}

fn get_specified_lists<'a>(
    args: &Arguments,
    tracker: &'a Tracker,
) -> Vec<(&'a MoneyList, &'static str)> {
    [
        (args.total, &tracker.total, "Total"),
        (args.incomes, &tracker.incomes, "Incomes"),
        (args.expenses, &tracker.expenses, "Expenses"),
    ]
    .iter()
    .filter_map(|(a, l, n)| if *a { Some((*l, *n)) } else { None })
    .collect::<Vec<_>>()
}

struct Table<'a, K>
where
    K: Eq + Hash + Display,
{
    columns: Vec<Column<'a>>,
    cells: HashMap<K, Vec<Cell>>,
    row_keys: Vec<RowKey<K>>,
    key_width: usize,
    key_prec: usize,
    pad: usize,
}

struct Column<'a> {
    header: Header<'a>,
    width: usize,
}

struct Header<'a> {
    name: &'a str,
    bold: bool,
    lines: Option<(usize, Vec<&'a str>)>,
}

enum RowKey<K> {
    Key(K),
    Dots,
}

#[derive(Clone, Default)]
enum Cell {
    Money {
        money: Money,
        color: bool,
    },
    Text {
        text: String,
        bold: bool,
    },
    #[default]
    Empty,
}

impl Cell {
    fn print(&self, col_width: usize) {
        match self {
            Self::Money { money, color } => {
                let mut fmt = String::with_capacity(col_width);
                match write!(&mut fmt, "{:>#col_width$.col_width$}", money) {
                    Ok(_) => {}
                    Err(e) => println!("{}", e),
                }
                match money {
                    m if *color && m.is_positive() => print!("{}", fmt.green()),
                    m if *color && m.is_negative() => print!("{}", fmt.bright_red()),
                    _ => print!("{}", fmt),
                }
            }
            Self::Text { text, bold } => {
                let mut text_to_print = String::with_capacity(col_width);
                match col_width {
                    w if w >= text.chars().count() => text_to_print.push_str(text),
                    w => {
                        let nth_char = text.char_indices().nth(w - 3).unwrap().0;
                        text_to_print.push_str(&text[..nth_char]);
                        text_to_print.push_str("...");
                    }
                };
                if *bold {
                    print!("{:col_width$}", text_to_print.bold());
                } else {
                    print!("{:col_width$}", text_to_print);
                }
            }
            Self::Empty => print!("{:col_width$}", ""),
        }
    }
}

impl RowKey<YearMonth> {
    fn from_year_months(year_months: &[YearMonth]) -> Vec<Self> {
        let mut row_keys: Vec<_> = year_months.iter().map(|ym| RowKey::Key(*ym)).collect();
        let mut i: usize = 0;
        while i < row_keys.len() - 1 {
            // the let statement and the if statement are separate because we need a
            // mutable borrow of row_keys to insert elements, which is not possible while
            // we are holding two immutable references to values inside of row_keys.
            let to_insert: Option<(YearMonth, YearMonth)> = match (&row_keys[i], &row_keys[i + 1]) {
                (RowKey::Key(current), RowKey::Key(next)) if next - current > 2 => {
                    Some((current.succ(), next.pred()))
                }
                _ => None,
            };
            if let Some((ym1, ym2)) = to_insert {
                row_keys.insert(i + 1, RowKey::Key(ym1));
                row_keys.insert(i + 2, RowKey::Dots);
                row_keys.insert(i + 3, RowKey::Key(ym2));
            }
            i += 1;
        }
        row_keys
    }
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
}

impl<'a, K> Table<'a, K>
where
    K: Clone + Eq + Hash + Display,
{
    fn new(row_keys: Vec<RowKey<K>>) -> Self {
        Self {
            columns: Vec::new(),
            cells: HashMap::new(),
            row_keys,
            key_width: 0,
            key_prec: 0,
            pad: TABLE_PADDING,
        }
    }

    fn add_column<F>(&mut self, name: &'a str, bold: bool, width: usize, cells: F)
    where
        F: Fn(&K) -> Cell,
    {
        self.columns.push(Column {
            header: Header::new(name, bold),
            width,
        });
        for key in &self.row_keys {
            if let RowKey::Key(k) = key {
                self.cells.entry(k.clone()).or_default().push(cells(k));
            }
        }
    }

    fn add_money_column<F>(&mut self, name: &'a str, bold: bool, entries: F, color: bool)
    where
        F: Fn(&K) -> Money,
    {
        self.add_column(name, bold, MONEY_COL_WIDTH, |k| Cell::Money {
            money: entries(k),
            color,
        });
    }

    fn add_text_column<F>(&mut self, name: &'a str, bold: bool, width: usize, cells: F)
    where
        F: Fn(&K) -> Option<String>,
    {
        self.add_column(name, bold, width, |k| match cells(k) {
            Some(text) => Cell::Text { text, bold: false },
            None => Cell::Empty,
        })
    }

    fn is_empty(&self) -> bool {
        self.columns.is_empty() || self.row_keys.is_empty()
    }

    fn print(&mut self) {
        if self.is_empty() {
            println!("[empty]");
            return;
        }
        // print headers
        let num_headers = self.columns.len();
        let num_header_rows = self
            .columns
            .iter_mut()
            .map(|c| c.header.get_num_lines(c.width))
            .max()
            .unwrap();
        for i in 0..num_header_rows {
            let key_width = self.key_width;
            let pad = self.pad;
            if key_width > 0 {
                print!("{:key_width$}{:pad$}", "", "");
            }
            for (j, column) in self.columns.iter_mut().enumerate() {
                let offset = num_header_rows - column.header.get_num_lines(column.width);
                column
                    .header
                    .print_line(i.wrapping_sub(offset), column.width);
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
                RowKey::Key(key) => {
                    let Some(row) = self.cells.get(key) else {
                        // this should never happen as values are always inserted for
                        // every row key
                        println!("[no row for key {}]", key);
                        continue;
                    };
                    self.print_row(row, Some(key));
                }
                RowKey::Dots => println!("..."),
            }
        }
    }

    fn print_row(&self, row: &[Cell], key: Option<&K>) {
        let key_width = self.key_width;
        let key_prec = self.key_prec;
        let pad = self.pad;
        if key_width > 0 {
            if key_prec > 0
                && let Some(key) = key
            {
                print!("{:key_width$.key_prec$}{:pad$}", key, "");
            } else {
                print!("{:key_width$}{:pad$}", "", "");
            }
        }
        for (j, cell) in row.iter().enumerate() {
            cell.print(self.columns[j].width);
            if j < row.len() - 1 {
                print!("{:pad$}", "");
            }
        }
        println!();
    }
}

impl<'a> Table<'a, YearMonth> {
    fn add_money_list(
        &mut self,
        money_list: &'a MoneyList,
        name: &'a str,
        show_categories: bool,
        sum_last: bool,
    ) {
        // headers
        if !sum_last {
            self.add_money_column(name, true, |ym| money_list.sum(ym), false);
        }
        if show_categories {
            for (i, category) in money_list.categories().iter().enumerate() {
                self.add_money_column(
                    &category.name,
                    false,
                    |ym| money_list.sum_category(ym, i),
                    false,
                );
            }
        }
        if sum_last {
            self.add_money_column(name, true, |ym| money_list.sum(ym), false);
        }
    }
}

impl<'a> Table<'a, usize> {
    fn with_num_rows(num_rows: usize) -> Self {
        Self::new((0..num_rows).map(RowKey::Key).collect())
    }

    fn add_money_list(
        &mut self,
        money_list: &MoneyList,
        entries: &[MoneyChange],
        name: &'a str,
        wide: bool,
    ) -> usize {
        let money_index = self.columns.len();
        // we do not use add_money_column because if there is no money in a cell, we want an empty
        // cell, not 0.00 €
        self.add_column(name, true, MONEY_COL_WIDTH, |i| {
            entries
                .get(*i)
                .map(|mc| Cell::Money {
                    money: mc.amount.eval(),
                    color: false,
                })
                .unwrap_or_default()
        });
        if money_list.allow_dates() {
            self.add_text_column("Date", false, DATE_COL_WIDTH, |i| {
                entries
                    .get(*i)
                    .and_then(|mc| mc.date)
                    .map(|d| d.to_string())
            });
        }
        let width = CATEGORY_COL_WIDTH * if wide { 2 } else { 1 };
        self.add_text_column("Category", false, width, |i| {
            entries
                .get(*i)
                .and_then(|mc| mc.category_id)
                .map(|i| money_list.categories()[i].name.clone())
        });
        // let min_desc_width = entries.iter().filter_map(|mc| mc.description.clone()).map(|d| d.len()).max().unwrap_or_default();
        // let col_width = usize::clamp(min_desc_width, "Description".len(), DESCRIPTION_COL_WIDTH);
        // only add a column for descriptions if any of the entries actually have a description
        let add_desc_col = entries.iter().find(|mc| mc.description.is_some()).is_some();
        if add_desc_col {
            let width = DESCRIPTION_COL_WIDTH * if wide { 2 } else { 1 };
            self.add_text_column("Description", false, width, |i| {
                entries.get(*i).and_then(|mc| mc.description.clone())
            });
        }
        money_index
    }
}
