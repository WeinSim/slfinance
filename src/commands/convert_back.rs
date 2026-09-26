use std::fs;

use crate::{
    expressions::{Expression, Sign, Term},
    money::{Category, Money, MoneyChange, MoneyList, Tracker, YearMonth},
};

const MONTH_NAMES_GERMAN: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];

pub fn convert_back(tracker: &mut Tracker, output_file: &str, german: bool) -> Result<(), String> {
    // first, make sure that all entries have a category by creating a "no-category" for entries
    // without a category
    for list in [
        &mut tracker.total,
        &mut tracker.incomes,
        &mut tracker.expenses,
    ] {
        let empty_id = list.categories().len();
        let mut used_empty = false;
        for vec in list.entries_mut().values_mut() {
            for mc in vec {
                match mc.category_id {
                    Some(_) => {}
                    None => {
                        used_empty = true;
                        mc.category_id = Some(empty_id);
                    }
                }
            }
        }
        // add this later because I cannot borrow list as mutable inside the for loop
        if used_empty {
            list.add_category(Category {
                name: "[no category]".to_owned(),
            })?;
        }
    }
    let mut tsv: Vec<Vec<String>> = Vec::new();
    // first row is for category names
    tsv.push(vec!["".to_owned()]);
    let year_months = tracker.get_year_months();
    for ym in &year_months {
        tsv.push(vec![format!(
            "{} {}",
            if german {
                MONTH_NAMES_GERMAN[ym.month.number_from_month() as usize - 1]
            } else {
                ym.month.name()
            },
            ym.year
        )])
    }
    add_list(
        &mut tsv,
        &tracker.total,
        &year_months,
        german,
        "Geld ges.",
        false,
    );
    let total_col = tsv[0].len() - 1;
    add_column(
        &mut tsv,
        "Tatsächliche Einnahmen / Ausgaben ges.",
        |i| format!("={0}{1}-{0}{2}", get_col_name(total_col), i + 3, i + 2),
        year_months.len(),
    );
    let change_col = total_col + 1;
    add_column(
        &mut tsv,
        "Unberücksichtigte Einnahmen / Ausgaben",
        |i| format!("={0}{1}-{2}{1}", get_col_name(change_col), i + 2, get_col_name(change_col + 2)),
        year_months.len(),
    );
    let incomes_col = change_col + 3;
    let expenses_col = incomes_col + tracker.incomes.categories().len() + 1;
    add_column(
        &mut tsv,
        "Einnahmen / Ausgaben ges.",
        |i| format!("={0}{1}-{2}{1}", get_col_name(incomes_col), i + 2, get_col_name(expenses_col)),
        year_months.len(),
    );
    add_list(
        &mut tsv,
        &tracker.incomes,
        &year_months,
        german,
        "Einnahmen ges.",
        true,
    );
    add_list(
        &mut tsv,
        &tracker.expenses,
        &year_months,
        german,
        "Ausgaben ges.",
        true,
    );
    let mut output_string = String::new();
    for row in tsv {
        for (i, col) in row.iter().enumerate() {
            output_string.push_str(col);
            if i < row.len() - 1 {
                output_string.push('\t');
            }
        }
        output_string.push('\n');
    }
    fs::write(output_file, output_string).map_err(|e| e.to_string())
}

fn add_list(
    tsv: &mut [Vec<String>],
    list: &MoneyList,
    year_months: &[YearMonth],
    german: bool,
    sum_name: &str,
    sum_first: bool,
) {
    let num_cats = list.categories().len();
    let start_col = tsv[1].len() + 1;
    if sum_first {
        add_sum_col(
            tsv,
            sum_name,
            start_col,
            start_col + num_cats - 1,
            year_months.len(),
        );
    }
    let empty_vec: Vec<MoneyChange> = Vec::new();
    let mut buckets: Vec<Vec<Vec<&Expression>>> =
        vec![vec![Vec::new(); num_cats]; year_months.len()];
    for (i, ym) in year_months.iter().enumerate() {
        let entries = list.entries().get(ym).unwrap_or(&empty_vec);
        for mc in entries {
            buckets[i][mc.category_id.unwrap_or(num_cats - 1)].push(&mc.amount);
        }
    }
    for cat in list.categories() {
        tsv[0].push(cat.name.to_owned());
    }
    // let start_index = tsv[1].len();
    for (r, row) in buckets.iter().enumerate() {
        for entries in row {
            let expr = match entries.len() {
                0 => Expression::Value(Money::default()),
                1 => entries[0].clone(),
                _ => Expression::Sum(
                    entries
                        .iter()
                        .map(|e| Term {
                            expression: (*e).clone(),
                            sign: Sign::Positive,
                        })
                        .collect(),
                ),
            };
            let expr_str = match expr {
                Expression::Value(_) => format!("{expr}"),
                _ => format!("={expr}"),
            };
            tsv[r + 1].push(if german {
                expr_str
                    .chars()
                    .map(|c| match c {
                        ',' => '.',
                        '.' => ',',
                        _ => c,
                    })
                    .collect::<String>()
            } else {
                expr_str
            });
        }
    }
    if !sum_first {
        let end_col = tsv[1].len() - 1;
        add_sum_col(
            tsv,
            sum_name,
            end_col - num_cats + 1,
            end_col,
            year_months.len(),
        );
    }
}

fn add_sum_col(
    tsv: &mut [Vec<String>],
    col_name: &str,
    start_col: usize,
    end_col: usize,
    num_rows: usize,
) {
    add_column(
        tsv,
        col_name,
        |i| {
            format!(
                "=SUM({0}{1}:{2}{1})",
                get_col_name(start_col),
                i + 2,
                get_col_name(end_col)
            )
        },
        num_rows,
    );
}

fn add_column<F>(tsv: &mut [Vec<String>], name: &str, entries: F, num_rows: usize)
where
    F: Fn(usize) -> String,
{
    tsv[0].push(name.to_owned());
    for i in 0..num_rows {
        tsv[i + 1].push(entries(i));
    }
}

fn get_col_name(col_index: usize) -> String {
    fn get_col_letter(i: usize) -> u8 {
        b'A' + i as u8
    }
    if col_index < 26 {
        let vec = vec![get_col_letter(col_index)];
        String::from_utf8(vec).unwrap()
    } else {
        let vec = vec![
            get_col_letter(col_index / 26 - 1),
            get_col_letter(col_index % 26),
        ];
        String::from_utf8(vec).unwrap()
    }
}
