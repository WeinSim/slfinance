use std::fs;

use chrono::Month;

use crate::{
    money::{Category, Money, MoneyChange, MoneyList, Tracker, YearMonth}, serial::save_file,
};

pub fn convert(input_file: &str, output_file: &str) -> Result<(), String> {
    let input = fs::read_to_string(input_file).map_err(|e| e.to_string())?;
    let mut tsv: Vec<Vec<&str>> = Vec::new();
    for line in input.lines() {
        tsv.push(line.split('\t').collect());
    }
    let mut tracker = Tracker::new();
    let num_total_categories = 8;
    let num_income_categories = 9;
    let num_expense_categories = 5;
    // start at 1 because cell (0, 0) is empty
    let mut i: usize = 1;
    add_list(&mut i, num_total_categories, &mut tracker.total, &tsv)?;
    i += 5;
    add_list(&mut i, num_income_categories, &mut tracker.incomes, &tsv)?;
    i += 1;
    add_list(&mut i, num_expense_categories, &mut tracker.expenses, &tsv)?;
    save_file(output_file, &tracker)
}

fn add_list(
    i: &mut usize,
    num: usize,
    list: &mut MoneyList,
    tsv: &Vec<Vec<&str>>,
) -> Result<(), String> {
    let i_initial = *i;
    for _ in 0..num {
        list.add_category(Category {
            name: tsv[0][*i].to_owned(),
        });
        *i += 1;
    }
    for row in &tsv[1..] {
        let mut ym_parts = row[0].split_whitespace();
        let month = match ym_parts.next().unwrap() {
            "Januar" => Month::January,
            "Februar" => Month::February,
            "März" => Month::March,
            "April" => Month::April,
            "Mai" => Month::May,
            "Juni" => Month::June,
            "Juli" => Month::July,
            "August" => Month::August,
            "September" => Month::September,
            "Oktober" => Month::October,
            "November" => Month::November,
            "Dezember" => Month::December,
            s => return Err(format!("Invalid month name: {s}")),
        };
        let year = ym_parts
            .next()
            .unwrap()
            .parse::<i32>()
            .expect("Second part of first cell should be a valid year number");
        let year_month = YearMonth { year, month };
        *i = i_initial;
        for j in 0..num {
            let amount = row[*i];
            let amount = amount.replace(&[',', '.', ' ', '€'], "");
            let money = if amount.is_empty() {
                Money::default()
            } else {
                Money {
                    cents: amount
                        .parse()
                        .expect(&format!("amount ('{}') should be a valid number", amount)),
                }
            };
            let entry = MoneyChange {
                amount: money,
                date: None,
                category_id: Some(j),
            };
            list.add_entry(year_month, entry)?;
            *i += 1;
        }
    }
    Ok(())
}
