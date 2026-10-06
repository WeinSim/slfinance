use chrono::{Datelike, NaiveDate};

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

pub fn export(tracker: &Tracker, output_file: &str, year_month: YearMonth) -> Result<(), String> {
    let mut template = Template::default();
    for (money_list, template_list, template_categories) in [
        (
            &tracker.total,
            &mut template.total,
            &mut template.total_categories,
        ),
        (
            &tracker.incomes,
            &mut template.incomes,
            &mut template.income_categories,
        ),
        (
            &tracker.expenses,
            &mut template.expenses,
            &mut template.expense_categories,
        ),
    ] {
        let Some(entries) = money_list.entries().get(&year_month) else {
            continue;
        };
        let mut categories: Vec<(usize, &str)> = Vec::new();
        for mc in entries {
            let cat_id = if let Some(cat_id) = mc.category_id {
                if let Some(id) = categories.iter().position(|(i, _)| *i == cat_id) {
                    Some(id)
                } else {
                    categories.push((cat_id, &money_list.categories()[cat_id].name));
                    Some(categories.len() - 1)
                }
            } else {
                None
            };
            let day = if let Some(date) = mc.date {
                let day = date.day() as i32;
                let month_len = year_month.num_days()? as i32;
                if day > month_len - 5 {
                    Some(-(month_len - day + 1))
                } else {
                    Some(day)
                }
            } else {
                None
            };
            template_list.push(Entry {
                amt: mc.amount.clone(),
                cat: cat_id,
                desc: if mc.description.is_empty() {
                    None
                } else {
                    Some(mc.description.clone())
                },
                day,
            });
        }
        *template_categories = categories.iter().map(|(_, c)| (*c).to_owned()).collect();
    }
    save_file(&template, output_file)
}

fn day_to_date(day: i32, year_month: YearMonth) -> Result<NaiveDate, String> {
    let num_days = year_month.num_days()? as i32;
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

fn save_file(template: &Template, filename: &str) -> Result<(), String> {
    let json = serde_json::to_string::<Template>(template)
        .map_err(|e| format!("unable to convert generated template to JSON: {e}"))?;
    fs::write(filename, json).map_err(|e| format!("unable to write to file file {filename}: {e}"))
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
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
