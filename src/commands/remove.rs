use crate::{
    commands::{MoneyListType, save_tracker},
    money::{Tracker, YearMonth},
};

pub fn remove(
    tracker: &mut Tracker,
    list_type: MoneyListType,
    year_month: YearMonth,
    index: usize,
) -> Result<(), String> {
    let list = tracker.get_mut_money_list(list_type);
    list.remove_entry(year_month, index)?;
    save_tracker(tracker)
}
