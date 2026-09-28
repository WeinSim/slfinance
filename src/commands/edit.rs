use crate::{
    TODAY,
    commands::{Arguments, MoneyListType},
    money::{Tracker, YearMonth},
};

pub fn edit(tracker: &mut Tracker, args: &Arguments, index: usize) -> Result<(), String> {
    let list = tracker.get_mut_money_list(
        args.get_unique_list_type()
            .unwrap_or(MoneyListType::Expenses),
    );
    let year_month = args
        .get_year_month()
        .unwrap_or(YearMonth::from_naive_date(*TODAY));
    if let Some(ref d) = args.description {
        list.set_entry_description(year_month, index, d.to_owned())?;
    }
    if let Some(ref c) = args.category {
        list.set_entry_category(year_month, index, c)?;
    }
    if let Some(ref a) = args.amount {
        list.set_entry_amount(year_month, index, a.clone())?;
    }
    if let Some(d) = args.date {
        list.set_entry_date(year_month, index, d)?;
    }
    Ok(())
}
