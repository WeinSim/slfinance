use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    iter::Sum,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use chrono::{Month, NaiveDate};

pub struct Tracker {
    total: HashMap<YearMonth, Vec<MoneySnapshot>>,
    total_categories: Vec<Category>,
    income: HashMap<YearMonth, Vec<MoneyChange>>,
    income_categories: Vec<Category>,
    expenses: HashMap<YearMonth, Vec<MoneyChange>>,
    expenses_categories: Vec<Category>,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            total: HashMap::new(),
            total_categories: Vec::new(),
            income: HashMap::new(),
            income_categories: Vec::new(),
            expenses: HashMap::new(),
            expenses_categories: Vec::new(),
        }
    }

    pub fn add_total(&mut self, year_month: YearMonth, snapshot: MoneySnapshot) {
        let vec_opt = self.total.get_mut(&year_month);
        let vec = match vec_opt {
            Some(v) => v,
            None => {
                self.total.insert(year_month, Vec::new());
                self.total.get_mut(&year_month).expect("map should contain the newly inserted vector")
            }
        };
        vec.push(snapshot);
    }

    pub fn add_total_category(&mut self, category: Category) {
        self.total_categories.push(category);
    }

    pub fn get_num_total_categories(&self) -> usize {
        self.total_categories.len()
    }

    pub fn get_total_sum(&self, year_month: &YearMonth) -> Money {
        match self.total.get(year_month) {
            Some(v) => v.iter().map(|ms| ms.amount).sum(),
            None => Money::default(),
        }
    }

    pub fn get_year_months(&self) -> Vec<YearMonth> {
        self.total
            .keys()
            .chain(self.income.keys())
            .chain(self.expenses.keys())
            .copied()
            .collect()
    }

    pub fn total_categories(&self) -> &Vec<Category> {
        &self.total_categories
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct YearMonth {
    pub year: i32,
    pub month: Month,
}

pub struct MoneyChange {
    amount: Money,
    sign: Sign,
    date: Option<NaiveDate>,
    category: Option<usize>,
}

pub struct MoneySnapshot {
    pub amount: Money,
    pub category: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct Money {
    pub cents: u64,
}

impl Money {
    pub fn default() -> Self {
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
        let mut sum = Self::default();
        for money in iter {
            sum += money;
        }
        sum
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

pub enum Sign {
    Positive = 1,
    Negative = -1,
}

pub struct Category {
    pub name: String,
}
