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
    category: Option<&str>,
    expression: &Expression,
) -> Result<(), String> {
    let list = tracker.get_mut_money_list(list_type);
    let cat_id = match category {
        Some(prefix) => Some(list.find_category_by_prefix(prefix)?),
        None => None,
    };
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
            category_id: cat_id,
            description: args.description.clone(),
        },
    )?;
    save_tracker(tracker)
}
