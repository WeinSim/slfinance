use chrono::NaiveDate;

use crate::{
    expressions::expr_serde,
    money::{MoneyChange, YearMonth},
};
use std::fs;

use crate::{expressions::Expression, money::Tracker};

pub fn import(
    tracker: &mut Tracker,
    input_file: &str,
    year_month: YearMonth,
) -> Result<(), String> {
    let template = load_file(input_file)?;
    for (list, entries, cat_names) in [
        (
            &mut tracker.total,
            &template.total,
            &template.total_categories,
        ),
        (
            &mut tracker.incomes,
            &template.incomes,
            &template.income_categories,
        ),
        (
            &mut tracker.expenses,
            &template.expenses,
            &template.expense_categories,
        ),
    ] {
        for entry in entries {
            let date = match entry.day {
                Some(day) => Some(day_to_date(day, year_month)?),
                _ => None,
            };
            let cat_name = entry.cat.and_then(|i| cat_names.get(i));
            let cat_id = match cat_name {
                Some(name) => Some(
                    list.find_category(name)
                        .ok_or_else(|| format!("unable to find category \"{name}\""))?,
                ),
                None => None,
            };
            list.add_entry(
                year_month,
                MoneyChange {
                    amount: entry.amt.clone(),
                    date,
                    category_id: cat_id,
                    description: entry.desc.clone().unwrap_or_default(),
                },
            )?;
        }
    }
    Ok(())
}

fn day_to_date(day: i32, year_month: YearMonth) -> Result<NaiveDate, String> {
    let num_days = year_month
        .num_days()
        .ok_or_else(|| format!("year out of range: {}", year_month.year))?
        as i32;
    let d = match day {
        d if d < 0 && -num_days <= d => num_days + d + 1,
        d if d > 0 => d,
        _ => return Err(format!("day out of range: {day}")),
    };
    NaiveDate::from_ymd_opt(
        year_month.year,
        year_month.month.number_from_month(),
        d as u32,
    )
    .ok_or(format!(
        "invalid date: day = {day}, month / year = {year_month}"
    ))
}

fn load_file(filename: &str) -> Result<Template, String> {
    let json =
        fs::read_to_string(filename).map_err(|e| format!("unable to open file {filename}: {e}"))?;
    serde_json::from_str::<Template>(&json)
        .map_err(|e| format!("unable to parse file {filename}: {e}"))
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Template {
    total: Vec<Entry>,
    total_categories: Vec<String>,
    incomes: Vec<Entry>,
    income_categories: Vec<String>,
    expenses: Vec<Entry>,
    expense_categories: Vec<String>,
}

#[serde_with::skip_serializing_none]
#[derive(serde::Deserialize, serde::Serialize)]
struct Entry {
    #[serde(with = "expr_serde")]
    amt: Expression,
    cat: Option<usize>,
    desc: Option<String>,
    day: Option<i32>,
}
