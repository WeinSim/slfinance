use std::fs;

use chrono::{Month, NaiveDate};

use crate::money::{Category, Money, MoneyChange, MoneyList, Sign, Tracker, YearMonth};

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
    incomes: Vec<YearMonthEntry>,
    income_categories: Vec<String>,
    expenses: Vec<YearMonthEntry>,
    expense_categories: Vec<String>,
}

impl SerialTracker {
    fn as_tracker(self) -> Result<Tracker, String> {
        let mut tracker = Tracker::new();
        Self::add_entries(&mut tracker.total, &self.total_categories, &self.total)?;
        Self::add_entries(&mut tracker.incomes, &self.income_categories, &self.incomes)?;
        Self::add_entries(&mut tracker.expenses, &self.expense_categories, &self.expenses)?;
        Ok(tracker)
    }

    fn add_entries(
        list: &mut MoneyList,
        categories: &Vec<String>,
        ym_entries: &Vec<YearMonthEntry>,
    ) -> Result<(), String> {
        for cat in categories {
            list.add_category(Category {
                name: cat.to_owned(),
            });
        }
        for ym_entry in ym_entries {
            let ym = YearMonth {
                year: ym_entry.year,
                month: ym_entry.month,
            };
            for entry in &ym_entry.entries {
                let (cents, sign) = if entry.amount < 0 {
                    (-entry.amount as u64, Sign::Negative)
                } else {
                    (entry.amount as u64, Sign::Positive)
                };
                let amount = Money { cents };
                list.add_entry(
                    ym,
                    MoneyChange {
                        amount,
                        sign,
                        category: entry.category,
                        date: entry.date,
                    },
                )?;
            }
        }
        Ok(())
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
