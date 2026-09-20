use chrono::{Datelike, Local};

use crate::{
    commands::{Arguments, MoneyListType, save_tracker},
    expressions::Expression,
    money::{MoneyChange, Tracker, YearMonth},
};

pub fn add(
    args: &Arguments,
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
    let today = Local::now().date_naive();
    let given_ym = if args.month.is_none() && args.year.is_none() {
        None
    } else {
        Some(YearMonth {
            year: args.year.unwrap_or(today.year()),
            month: args
                .month
                .unwrap_or(YearMonth::month_from_naive_date(today)),
        })
    };
    let date = args.date.unwrap_or(match given_ym {
        Some(_) => None,
        None if list_type == MoneyListType::Total => None,
        None => Some(today),
    });
    list.add_entry(
        given_ym.unwrap_or(YearMonth::from_naive_date(date.unwrap_or(today))),
        MoneyChange {
            amount: expression.clone(),
            date,
            category_id: Some(cat_id),
            description: args.description.clone(),
        },
    )?;
    save_tracker(&tracker)
}
