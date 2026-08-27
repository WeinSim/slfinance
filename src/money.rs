use std::fmt::{Display, Formatter};

use chrono::{Datelike, Month, NaiveDate};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Tracker {
    pub total: Vec<MoneySnapshot>,
    pub income: Vec<MoneyChange>,
    pub expenses: Vec<MoneyChange>,
}

impl Tracker {
    fn new() -> Self {
        Self {
            total: vec![],
            income: vec![],
            expenses: vec![],
        }
    }

    // pub fn get_min_year(&self) -> Option<i32> {
    //     match self.get_years().iter().min() {
    //         Some(&i) => Some(i),
    //         None => None,
    //     }
    // }

    // pub fn get_max_year(&self) -> Option<i32> {
    //     match self.get_years().iter().max() {
    //         Some(&i) => Some(i),
    //         None => None,
    //     }
    // }

    pub fn get_years(&self) -> Vec<i32> {
        self.total
            .iter()
            .map(|ms| ms.year)
            .chain(self.income.iter().map(|mc| mc.date.year()))
            .chain(self.expenses.iter().map(|mc| mc.date.year()))
            .collect()
    }
}

#[derive(Deserialize)]
pub struct MoneyChange {
    pub amount: Money,
    pub sign: Sign,
    pub date: NaiveDate,
}

#[derive(Deserialize)]
pub struct MoneySnapshot {
    pub amount: Money,
    pub month: Month,
    pub year: i32,
}

#[derive(Deserialize)]
pub struct Money {
    pub cents: u64,
}

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let major = self.cents / 100;
        let minor = self.cents % 100;
        match f.width() {
            Some(width) => {
                let major_width = width - 4;
                write!(f, "{major:major_width$}.{minor:02}€")
            }
            None => write!(f, "{major}.{minor:02}€"),
        }
    }
}

#[derive(Deserialize)]
pub enum Sign {
    Positive = 1,
    Negative = -1,
}
