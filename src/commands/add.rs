use chrono::Local;

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
        MoneyListType::Incomes => &mut tracker.incomes,
        MoneyListType::Expenses => &mut tracker.expenses,
    };
    let cat_id = list.find_or_create_category(cat_name);
    let given_ym = args.get_year_month();
    let today = Local::now().date_naive();
    let date = args.date.unwrap_or(match given_ym {
        Some(_) => None,
        None if list.allow_dates() => Some(today),
        None => None,
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
    save_tracker(tracker)
}
