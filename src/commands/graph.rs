use owo_colors::OwoColorize;
use std::{collections::HashMap, sync::LazyLock};

use crate::{
    commands::{
        Arguments,
        list::{RowKey, get_specified_lists},
    },
    money::{Money, Tracker, YearMonth},
};

static DISPLAY_VALUES: LazyLock<Vec<Money>> = LazyLock::new(get_display_values);

pub fn graph(tracker: &Tracker, args: &Arguments) -> Result<(), String> {
    // TODO: this would select all lists if none is specified. we want only the total in this case
    for (list, name) in get_specified_lists(args, tracker) {
        let graph = Graph::new(&tracker.get_year_months(), |ym| {
            list.entries()
                .get(ym)
                .unwrap()
                .iter()
                .map(|mc| mc.amount.eval())
                .sum()
        });
        println!("{}", name.bold());
        graph.print(13, 3, 2, 1, 40);
    }
    Ok(())
}

struct Graph {
    keys: Vec<RowKey<YearMonth>>,
    values: HashMap<YearMonth, Money>,
}

impl Graph {
    fn new<F>(year_months: &[YearMonth], values: F) -> Self
    where
        F: Fn(&YearMonth) -> Money,
    {
        let mut map = HashMap::new();
        let keys = RowKey::from_year_months(year_months);
        for ym in year_months {
            map.insert(*ym, values(ym));
        }
        Self { keys, values: map }
    }

    fn print(
        &self,
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
        let max_money = *self.values.values().max().unwrap();
        let scale = max_money.cents / height as i64;
        let min_step_size = Money { cents: scale * 8 };
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
                let c = match key {
                    RowKey::Key(ym) if *self.values.get(ym).unwrap() > money => '#',
                    _ => ' ',
                };
                print!(
                    "{}{:pad$}",
                    std::iter::repeat_n(c, col_width).collect::<String>(),
                    "",
                );
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
