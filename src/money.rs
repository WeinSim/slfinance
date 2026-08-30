use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    iter::Sum,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use chrono::{Month, NaiveDate};

pub struct Tracker {
    pub total: MoneyList,
    pub incomes: MoneyList,
    pub expenses: MoneyList,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            total: MoneyList::new(false, false),
            incomes: MoneyList::new(true, true),
            expenses: MoneyList::new(true, true),
        }
    }

    pub fn get_year_months(&self) -> Vec<YearMonth> {
        let mut vec: Vec<YearMonth> = vec![];
        vec.append(&mut self.total.get_year_months());
        vec.append(&mut self.incomes.get_year_months());
        vec.append(&mut self.expenses.get_year_months());
        vec
    }
}

pub struct MoneyList {
    entries: HashMap<YearMonth, Vec<MoneyChange>>,
    categories: Vec<Category>,
    allow_negatives: bool,
    allow_dates: bool,
}

impl MoneyList {
    fn new(allow_negatives: bool, allow_dates: bool) -> Self {
        Self {
            entries: HashMap::new(),
            categories: Vec::new(),
            allow_negatives,
            allow_dates,
        }
    }

    pub fn add_entry(&mut self, year_month: YearMonth, entry: MoneyChange) -> Result<(), String> {
        if !self.allow_negatives && entry.sign == Sign::Negative {
            return Err("A 'total' value cannot be negative".to_owned());
        }
        if !self.allow_dates && entry.date.is_some() {
            return Err("A 'total' entry cannot have a date associated with it".to_owned());
        }
        match entry.category {
            Some(i) if i >= self.categories.len() => {
                return Err(format!(
                    "Index out of range (index={}, len={}",
                    i,
                    self.categories.len(),
                ));
            }
            _ => {}
        }
        self.entries.entry(year_month).or_default().push(entry);
        Ok(())
    }

    pub fn sum(&self, year_month: &YearMonth) -> Money {
        self.entries
            .get(year_month)
            .and_then(|v| Some(v.iter().map(|mc| mc.amount).sum()))
            .unwrap_or_default()
    }

    pub fn add_category(&mut self, category: Category) {
        self.categories.push(category);
    }

    pub fn categories(&self) -> &Vec<Category> {
        &self.categories
    }

    pub fn get_year_months(&self) -> Vec<YearMonth> {
        self.entries.keys().copied().collect()
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct YearMonth {
    pub year: i32,
    pub month: Month,
}

pub struct MoneyChange {
    pub amount: Money,
    pub sign: Sign,
    pub date: Option<NaiveDate>,
    pub category: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct Money {
    pub cents: u64,
}

impl Default for Money {
    fn default() -> Self {
        Self { cents: 0 }
    }
}

impl Add<Money> for Money {
    type Output = Self;
    fn add(self, rhs: Money) -> Self::Output {
        Self {
            cents: self.cents + rhs.cents,
        }
    }
}

impl Sub<Money> for Money {
    type Output = Self;
    fn sub(self, rhs: Money) -> Self::Output {
        Self {
            cents: self.cents - rhs.cents,
        }
    }
}

impl AddAssign<Money> for Money {
    fn add_assign(&mut self, rhs: Money) {
        self.cents += rhs.cents;
    }
}

impl SubAssign<Money> for Money {
    fn sub_assign(&mut self, rhs: Money) {
        self.cents -= rhs.cents;
    }
}

impl Sum<Money> for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Self {
        Self {
            cents: iter.map(|m| m.cents).sum(),
        }
    }
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

#[derive(PartialEq, Eq)]
pub enum Sign {
    Positive = 1,
    Negative = -1,
}

pub struct Category {
    pub name: String,
}
