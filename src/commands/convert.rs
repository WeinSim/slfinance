use std::fs;

use chrono::Month;

use crate::{
    expressions::{Expression, Term},
    money::{Category, Money, MoneyChange, MoneyList, Tracker, YearMonth},
    serial::save_file,
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
    // start at 1 because that's the first 'total' column
    let mut i: usize = 1;
    add_list(
        &mut i,
        num_total_categories,
        &mut tracker.total,
        &tsv,
        false,
    )?;
    i += 5; // skip all summarizing columns between 'total' and 'incomes'
    add_list(
        &mut i,
        num_income_categories,
        &mut tracker.incomes,
        &tsv,
        true,
    )?;
    i += 1; // skip 'expenses' column
    add_list(
        &mut i,
        num_expense_categories,
        &mut tracker.expenses,
        &tsv,
        true,
    )?;
    save_file(output_file, &tracker)?;
    println!("Successfully converted {input_file} to {output_file}");
    Ok(())
}

fn add_list(
    i: &mut usize,
    num: usize,
    list: &mut MoneyList,
    tsv: &Vec<Vec<&str>>,
    split_terms: bool,
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
            let cell = row[*i];
            let amounts: Vec<Money> = if cell.starts_with('=') {
                match Expression::parse(&cell[1..])? {
                    Expression::Sum(terms) if split_terms => terms.iter().map(Term::eval).collect(),
                    e => vec![e.eval()],
                }
            } else {
                vec![Money {
                    cents: cell
                        .replace(&[',', '.', ' ', '€'], "")
                        .parse::<i64>()
                        .map_err(|e| e.to_string())?,
                }]
            };
            for amount in amounts {
                if amount.cents == 0 {
                    continue;
                }
                let entry = MoneyChange {
                    amount,
                    date: None,
                    category_id: Some(j),
                    description: None,
                };
                list.add_entry(year_month, entry)?;
            }
            *i += 1;
        }
    }
    Ok(())
}
