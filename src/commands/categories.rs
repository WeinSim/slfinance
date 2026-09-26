use crate::{
    commands::MoneyListType,
    money::{Category, Tracker},
};

pub fn add_category(
    tracker: &mut Tracker,
    name: &str,
    list_type: MoneyListType,
) -> Result<(), String> {
    tracker
        .get_mut_money_list(list_type)
        .add_category(Category {
            name: name.to_owned(),
        })
}

pub fn remove_category(
    tracker: &mut Tracker,
    prefix: &str,
    list_type: MoneyListType,
) -> Result<(), String> {
    let list = tracker.get_mut_money_list(list_type);
    let cat_id = list.find_category_by_prefix(prefix)?;
    println!("Removing category '{}'", list.categories()[cat_id].name);
    list.remove_category(cat_id);
    Ok(())
}
