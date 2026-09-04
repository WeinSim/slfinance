use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{Display, Formatter, Write},
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

    pub fn get_total_change(&self, year_month: &YearMonth) -> Money {
        self.total.sum(&year_month.succ()) - self.total.sum(year_month)
    }

    pub fn get_expected_total_change(&self, year_month: &YearMonth) -> Money {
        self.incomes.sum(year_month) - self.expenses.sum(year_month)
    }

    pub fn get_diff_total_change(&self, year_month: &YearMonth) -> Money {
        self.get_total_change(year_month) - self.get_expected_total_change(year_month)
    }

    pub fn get_year_months(&self) -> Vec<YearMonth> {
        let mut vec: Vec<YearMonth> = Vec::new();
        vec.append(&mut self.total.get_year_months());
        vec.append(&mut self.incomes.get_year_months());
        vec.append(&mut self.expenses.get_year_months());
        vec.sort();
        vec.dedup();
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
        if !self.allow_negatives && entry.amount.is_negative() {
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
            .map_or_default(|v| v.iter().map(|mc| mc.amount).sum())
    }

    pub fn sum_category(&self, year_month: &YearMonth, category: usize) -> Money {
        let Some(entries) = self.entries.get(year_month) else {
            return Money::default();
        };
        entries
            .iter()
            .filter(|e| e.category.is_some_and(|i| i == category))
            .map(|mc| mc.amount)
            .sum()
    }

    pub fn add_category(&mut self, category: Category) {
        self.categories.push(category);
    }

    pub fn categories(&self) -> &Vec<Category> {
        &self.categories
    }

    pub fn entries(&self) -> &HashMap<YearMonth, Vec<MoneyChange>> {
        &self.entries
    }

    pub fn get_year_months(&self) -> Vec<YearMonth> {
        let mut vec: Vec<YearMonth> = self.entries.keys().copied().collect();
        vec.sort();
        vec
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct YearMonth {
    pub year: i32,
    pub month: Month,
}

impl YearMonth {
    pub fn succ(&self) -> Self {
        Self {
            year: if self.month == Month::December {
                self.year + 1
            } else {
                self.year
            },
            month: self.month.succ(),
        }
    }
}

impl Ord for YearMonth {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.year.partial_cmp(&other.year) {
            Some(Ordering::Equal) | None => {}
            Some(o) => {
                return o;
            }
        }
        self.month.cmp(&other.month)
    }
}

impl PartialOrd for YearMonth {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MoneyChange {
    pub amount: Money,
    pub date: Option<NaiveDate>,
    pub category: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct Money {
    pub cents: i64,
}

impl Money {
    pub fn is_positive(&self) -> bool {
        self.cents > 0
    }

    pub fn is_negative(&self) -> bool {
        self.cents < 0
    }
}

impl Default for Money {
    fn default() -> Self {
        Self { cents: 0 }
    }
}

impl PartialEq for Money {
    fn eq(&self, other: &Self) -> bool {
        self.cents == other.cents
    }
}

impl Eq for Money {}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cents.cmp(&other.cents)
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

// The following format arguments are considered:
// width, fill, alignment, sign (+) and alternate(#)
// With the alternate flag (#), groups of 3 digits are separated by commas
// (e.g. 1,234.45€)
impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let major = self.cents / 100;
        let minor = self.cents % 100;
        let mut major_str = major.to_string();
        // split groups of 3 digits with commas
        if f.alternate() {
            let mut buf = String::new();
            let mut start: usize = 0;
            let mut end: usize = match major_str.len() % 3 {
                0 => 3,
                i => i,
            };
            loop {
                buf.push_str(&major_str[start..end]);
                if end < major_str.len() - 1 {
                    buf.push(',');
                } else {
                    break;
                }
                start = end;
                end = start + 3;
            }
            major_str = buf;
        }
        let mut base_str = String::with_capacity(major_str.capacity() + 1);
        if f.sign_plus() {
            base_str.push(if self.is_negative() { '-' } else { '+' });
        }
        base_str.push_str(&major_str);
        write!(&mut base_str, ".{minor:02}€")?;
        // let align = f.align().unwrap_or(Alignment::Right);
        // this should automatically account for width, fill and alignment.
        f.pad(&base_str)
    }
}

pub struct Category {
    pub name: String,
}
