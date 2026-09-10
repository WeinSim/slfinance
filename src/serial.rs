use std::fs;

use chrono::{Month, NaiveDate};

use crate::money::{Category, Money, MoneyChange, MoneyList, Tracker, YearMonth};

pub fn load_file(filename: &str) -> Result<Tracker, String> {
    let json =
        fs::read_to_string(filename).map_err(|e| format!("Unable to open file {filename}: {e}"))?;
    serde_json::from_str::<SerialTracker>(&json)
        .map_err(|e| format!("Unable to parse file {filename}: {e}"))?
        .as_tracker()
}

pub fn save_file(filename: &str, tracker: &Tracker) -> Result<(), String> {
    let json =
        serde_json::to_string(&SerialTracker::from_tracker(tracker)).map_err(|e| e.to_string())?;
    fs::write(filename, json).map_err(|e| e.to_string())
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
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
        Self::add_tracker_entries(&mut tracker.total, &self.total_categories, &self.total)?;
        Self::add_tracker_entries(&mut tracker.incomes, &self.income_categories, &self.incomes)?;
        Self::add_tracker_entries(
            &mut tracker.expenses,
            &self.expense_categories,
            &self.expenses,
        )?;
        Ok(tracker)
    }

    fn add_tracker_entries(
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
                let amount = Money {
                    cents: entry.amount,
                };
                list.add_entry(
                    ym,
                    MoneyChange {
                        amount,
                        category_id: entry.category,
                        date: entry.date,
                    },
                )?;
            }
        }
        Ok(())
    }

    fn from_tracker(tracker: &Tracker) -> Self {
        let mut serial = Self::default();
        Self::add_serial_entries(
            &mut serial.total,
            &tracker.total,
            &mut serial.total_categories,
        );
        Self::add_serial_entries(
            &mut serial.incomes,
            &tracker.incomes,
            &mut serial.income_categories,
        );
        Self::add_serial_entries(
            &mut serial.expenses,
            &tracker.expenses,
            &mut serial.expense_categories,
        );
        serial
    }

    fn add_serial_entries(
        serial_list: &mut Vec<YearMonthEntry>,
        money_list: &MoneyList,
        categories: &mut Vec<String>,
    ) {
        let mut entries = money_list.entries().into_iter().collect::<Vec<_>>();
        entries.sort_by(|e1, e2| e1.0.cmp(e2.0));
        for (ym, entries) in entries {
            serial_list.push(YearMonthEntry {
                month: ym.month,
                year: ym.year,
                entries: entries
                    .iter()
                    .map(|mc| Entry {
                        category: mc.category_id,
                        amount: mc.amount.cents,
                        date: mc.date,
                    })
                    .collect(),
            })
        }
        for category in money_list.categories() {
            categories.push(category.name.to_owned());
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct YearMonthEntry {
    month: Month,
    year: i32,
    entries: Vec<Entry>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Entry {
    category: Option<usize>,
    amount: i64,
    date: Option<NaiveDate>,
}
