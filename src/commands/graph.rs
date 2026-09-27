use owo_colors::OwoColorize;
use std::{collections::HashMap, fmt::Display, sync::LazyLock};

use crate::{
    commands::{
        Arguments,
        list::{RowKey, get_specified_lists},
    },
    money::{Money, MoneyList, Tracker, YearMonth},
};

static DISPLAY_VALUES: LazyLock<Vec<Money>> = LazyLock::new(get_display_values);

pub fn graph(tracker: &Tracker, args: &Arguments) -> Result<(), String> {
    // TODO: this would select all lists if none is specified. we want only the total in this case
    for (list, name) in get_specified_lists(args, tracker) {
        let graph = Graph::new(list, args.year);
        println!("{}", name.bold());
        graph.print(args.show_categories.is_some(), 13, 3, 2, 1, 40);
    }
    Ok(())
}

struct Graph<'a> {
    keys: Vec<RowKey<YearMonth>>,
    values: HashMap<YearMonth, (Money, Vec<Money>)>,
    category_names: Vec<&'a String>,
}

impl<'a> Graph<'a> {
    fn new(list: &'a MoneyList, year: Option<i32>) -> Self {
        let mut map = HashMap::new();
        let year_months: Vec<_> = list
            .get_year_months()
            .iter()
            .filter(|ym| year.is_none_or(|y| y == ym.year))
            .copied()
            .collect();
        let keys = RowKey::from_year_months(&year_months);
        for ym in year_months {
            let entries = list.entries().get(&ym).unwrap();
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
            map.insert(ym, (total, cat_totals));
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
                Self::print_colored(cat, self.category_names.len() - i);
                // print!("{}", if (i + 1) % 4 == 0 { "\n" } else { "\t" });
                println!();
            }
            // println!();
        }
        let max_money = *self.values.values().map(|(money, _)| money).max().unwrap();
        let scale = max_money.cents / height as i64;
        let min_step_size = Money { cents: scale * 5 };
        let step_size = DISPLAY_VALUES[DISPLAY_VALUES.partition_point(|&val| val <= min_step_size)];
        let mut last_display_value = Money {
            cents: (max_money.cents / step_size.cents + 1) * step_size.cents,
        };
        for row in (0..height).rev() {
            let money = Money {
                cents: row as i64 * scale,
            };
            let print_money = money <= last_display_value - step_size;
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
                        let c = if money <= *m { '#' } else { ' ' };
                        (c, cat.partition_point(|m| *m <= money) + 1)
                    }
                    _ => (' ', 0),
                };
                let to_print = std::iter::repeat_n(c, col_width).collect::<String>();
                if show_categories {
                    Self::print_colored(to_print, color_id);
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
            1 => print!("{}", t.bright_red()),
            2 => print!("{}", t.bright_green()),
            3 => print!("{}", t.bright_blue()),
            4 => print!("{}", t.bright_yellow()),
            5 => print!("{}", t.bright_magenta()),
            6 => print!("{}", t.bright_cyan()),
            7 => print!("{}", t.red()),
            8 => print!("{}", t.green()),
            9 => print!("{}", t.blue()),
            10 => print!("{}", t.yellow()),
            11 => print!("{}", t.magenta()),
            12 => print!("{}", t.cyan()),
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
