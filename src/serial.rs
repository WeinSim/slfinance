use std::fs;

use chrono::{Month, NaiveDate};

use crate::money::{Category, Money, MoneySnapshot, Tracker, YearMonth};

pub fn load_file(filename: &str) -> Result<Tracker, String> {
    let json = fs::read_to_string(filename).map_err(|e| e.to_string())?;
    serde_json::from_str::<SerialTracker>(&json)
        .map_err(|e| e.to_string())?
        .as_tracker()
}

#[derive(serde::Deserialize)]
struct SerialTracker {
    total: Vec<YearMonthEntry>,
    total_categories: Vec<String>,
    income: Vec<YearMonthEntry>,
    income_categories: Vec<String>,
    expenses: Vec<YearMonthEntry>,
    expenses_categories: Vec<String>,
}

impl SerialTracker {
    fn as_tracker(self) -> Result<Tracker, String> {
        let mut tracker = Tracker::new();
        for cat in self.total_categories {
            tracker.add_total_category(Category { name: cat });
        }
        for ym_entry in self.total {
            let ym = YearMonth {
                year: ym_entry.year,
                month: ym_entry.month,
            };
            for entry in ym_entry.entries {
                if entry.amount < 0 {
                    todo!("print error here")
                }
                let amount = Money {
                    cents: entry.amount as u64,
                };
                let category = match entry.category {
                    Some(i) if i < tracker.get_num_total_categories() => Some(i),
                    _ => None,
                };
                tracker.add_total(ym, MoneySnapshot { amount, category });
            }
        }
        Ok(tracker)
    }
}

#[derive(serde::Deserialize)]
struct YearMonthEntry {
    month: Month,
    year: i32,
    entries: Vec<Entry>,
}

#[derive(serde::Deserialize)]
struct Entry {
    category: Option<usize>,
    amount: i64,
    date: Option<NaiveDate>,
}
