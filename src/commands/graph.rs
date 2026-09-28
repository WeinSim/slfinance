use owo_colors::OwoColorize;
use std::{collections::HashMap, fmt::Display, sync::LazyLock};

use crate::{
    commands::{Arguments, MoneyListType, list::RowKey},
    money::{Money, MoneyChange, MoneyList, Tracker, YearMonth},
};

static DISPLAY_VALUES: LazyLock<Vec<Money>> = LazyLock::new(get_display_values);

pub fn graph(tracker: &Tracker, args: &Arguments) -> Result<(), String> {
    for (list, name) in
        tracker.get_money_lists(&args.get_list_types_or(vec![MoneyListType::Expenses]))
    {
        let year_months: Vec<_> = tracker
            .get_year_months()
            .iter()
            .filter(|ym| args.year.is_none_or(|y| y == ym.year))
            .copied()
            .collect();
        let graph = Graph::new(list, &year_months);
        println!("{}", name.bold());
        graph.print(args.show_categories, 13, 3, 2, 1, 40);
    }
    Ok(())
}

struct Graph<'a> {
    keys: Vec<RowKey<YearMonth>>,
    values: HashMap<YearMonth, (Money, Vec<Money>)>,
    category_names: Vec<&'a String>,
}

impl<'a> Graph<'a> {
    fn new(list: &'a MoneyList, year_months: &[YearMonth]) -> Self {
        let mut map = HashMap::new();
        let keys = RowKey::from_year_months(year_months);
        let empty_vec: Vec<MoneyChange> = Vec::new();
        for key in &keys {
            let RowKey::Key(ym) = key else {
                continue;
            };
            let entries = list.entries().get(ym).unwrap_or(&empty_vec);
            let total = entries.iter().map(|mc| mc.amount.eval()).sum();
            let cat_totals = (0..list.categories().len())
                .map(|i| {
                    entries
                        .iter()
                        .filter(|mc| mc.category_id.is_some_and(|id| id <= i))
                        .map(|mc| mc.amount.eval())
                        .sum()
                })
                .collect();
            map.insert(*ym, (total, cat_totals));
        }
        let category_names = list.categories().iter().map(|c| &c.name).collect();
        Self {
            keys,
            values: map,
            category_names,
        }
    }

    fn print(
        &self,
        show_categories: bool,
        key_width: usize,
        pad_left: usize,
        col_width: usize,
        pad: usize,
        height: usize,
    ) {
        if self.values.is_empty() {
            println!("[empty]");
            return;
        }
        if show_categories {
            for (i, cat) in self.category_names.iter().rev().enumerate() {
                print!("{:pad_left$}", "");
                Self::print_colored(cat, self.category_names.len() - i - 1);
                println!();
            }
        }
        let max_money = *self.values.values().map(|(money, _)| money).max().unwrap();
        let scale = max_money.cents / height as i64;
        let min_step_size = Money { cents: scale * 5 };
        let step_size = DISPLAY_VALUES[DISPLAY_VALUES.partition_point(|&val| val <= min_step_size)];
        let mut last_display_value = Money {
            cents: (max_money.cents / step_size.cents + 1) * step_size.cents,
        };
        for row in (0..height).rev() {
            let row_money = Money {
                cents: row as i64 * scale,
            };
            let print_money = row_money <= last_display_value - step_size;
            if print_money {
                last_display_value -= step_size;
                print!("{last_display_value:>#key_width$}{:pad_left$}", "");
            } else {
                print!("{:key_width$}{:pad_left$}", "", "");
            }
            for key in &self.keys {
                let (c, color_id) = match key {
                    RowKey::Key(ym) => {
                        let (m, cat) = self.values.get(ym).unwrap();
                        if *m > row_money {
                            let part_point = cat.partition_point(|m| *m <= row_money);
                            let cat_id = if part_point == cat.len() {
                                usize::MAX
                            } else {
                                part_point
                            };
                            ('#', cat_id)
                        } else {
                            (' ', usize::MAX)
                        }
                    }
                    _ => (' ', 0),
                };
                let to_print = std::iter::repeat_n(c, col_width).collect::<String>();
                if show_categories {
                    Self::print_colored(to_print, color_id);
                    // Self::print_colored(color_id, color_id);
                    // print!("{}", color_id);
                } else {
                    print!("{}", to_print);
                }
                print!("{:pad$}", "",);
            }
            println!();
        }
        println!();
        print!("{:key_width$}{:pad_left$}", "", "");
        for key in &self.keys {
            print!("{:.col_width$}{:pad$}", key.to_string(), "");
        }
        println!();
    }

    fn print_colored<T>(t: T, color_id: usize)
    where
        T: Display,
    {
        match color_id {
            0 => print!("{}", t.bright_red()),
            1 => print!("{}", t.bright_green()),
            2 => print!("{}", t.bright_blue()),
            3 => print!("{}", t.bright_yellow()),
            4 => print!("{}", t.bright_magenta()),
            5 => print!("{}", t.bright_cyan()),
            6 => print!("{}", t.red()),
            7 => print!("{}", t.green()),
            8 => print!("{}", t.blue()),
            9 => print!("{}", t.yellow()),
            10 => print!("{}", t.magenta()),
            11 => print!("{}", t.cyan()),
            _ => print!("{}", t),
        }
    }
}

fn get_display_values() -> Vec<Money> {
    let mut ret = Vec::new();
    let mut base: i64 = 1;
    while base <= i64::MAX / 10 {
        ret.push(base);
        ret.push(2 * base);
        ret.push(5 * base);
        base *= 10;
    }
    ret.iter().map(|&cents| Money { cents }).collect()
}
