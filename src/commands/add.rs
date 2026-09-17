use chrono::Local;

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
    let list = match list_type {
        MoneyListType::Total => &mut tracker.total,
        MoneyListType::Income => &mut tracker.incomes,
        MoneyListType::Expense => &mut tracker.expenses,
    };
    let cat_id = list.find_or_create_category(cat_name);
    let date = args.iter().find_map(|a| match a {
        Argument::Date(d) => Some(d.to_owned()),
        _ => None,
    });
    let today = Local::now().date_naive();
    let ym = YearMonth::from_naive_date(date.unwrap_or(today));
    list.add_entry(
        ym,
        MoneyChange {
            amount: expression.clone(),
            date: if date.is_none() && list_type != MoneyListType::Total {
                Some(today)
            } else {
                date
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
