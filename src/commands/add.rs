use chrono::{Datelike, Local, Month};

use crate::{
    commands::{Argument, MoneyListType, save_tracker},
    expressions::Expression,
    money::{MoneyChange, Tracker, YearMonth},
};

pub fn add(
    args: &[Argument],
    tracker: &mut Tracker,
    list_type: MoneyListType,
    cat_name: &str,
    expression: &Expression,
) -> Result<(), String> {
    let date = Local::now().date_naive();
    let list = match list_type {
        MoneyListType::Total => &mut tracker.total,
        MoneyListType::Income => &mut tracker.incomes,
        MoneyListType::Expense => &mut tracker.expenses,
    };
    let cat_id = list.find_or_create_category(cat_name);
    list.add_entry(
        YearMonth {
            year: date.year(),
            month: Month::try_from(date.month() as u8).unwrap(),
        },
        MoneyChange {
            amount: expression.eval(),
            date: match list_type {
                MoneyListType::Total => None,
                _ => Some(date),
            },
            category_id: Some(cat_id),
            description: args.iter().find_map(|a| match a {
                Argument::Description(d) => Some(d.to_owned()),
                _ => None,
            }),
        },
    )?;
    save_tracker(&tracker)
}
