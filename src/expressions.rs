use std::{fmt::Display, str::FromStr};

use crate::money::Money;

#[derive(Clone)]
pub enum Expression {
    Value(Money),
    Sum(Vec<Term>),
    Multiplication(Box<Expression>, i64),
    Division(Box<Expression>, i64),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Positive = 1,
    Negative = -1,
}

#[derive(Clone)]
pub struct Term {
    pub expression: Expression,
    pub sign: Sign,
}

impl Term {
    pub fn eval(&self) -> Money {
        self.expression.eval().mult(self.sign as i64)
    }
}

impl Expression {
    pub fn eval(&self) -> Money {
        match self {
            Self::Value(m) => *m,
            Self::Sum(v) => v.iter().map(Term::eval).sum(),
            Self::Multiplication(e, f) => e.eval().mult(*f),
            Self::Division(e, d) => e.eval().div(*d),
        }
    }
}

impl FromStr for Expression {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = CharIter::new(s.chars().filter(|c| !c.is_whitespace()).collect());
        if !iter.has_next() {
            return Err("cannot parse an empty string".to_owned());
        }
        parse_expression(&mut iter)
    }
}

// this function parses either to the end of the iterator or to the first closing
// paretheses that it encounters
fn parse_expression(iter: &mut CharIter) -> Result<Expression, String> {
    let mut terms: Vec<Term> = Vec::new();
    while let Some(c) = iter.next() {
        match c {
            '+' | '-' => {
                terms.push(Term {
                    expression: parse_term(iter)?,
                    sign: if c == '+' {
                        Sign::Positive
                    } else {
                        Sign::Negative
                    },
                });
            }
            '*' | '/' => {
                let last = terms.pop().unwrap();
                let int = parse_int(iter)?;
                let expression = if c == '*' {
                    Expression::Multiplication(Box::new(last.expression), int)
                } else {
                    Expression::Division(Box::new(last.expression), int)
                };
                terms.push(Term {
                    expression,
                    sign: last.sign,
                });
            }
            ')' => break,
            _ => {
                iter.go_back();
                terms.push(Term {
                    expression: parse_term(iter)?,
                    sign: Sign::Positive,
                });
            } // c => return Err(format!("unexpected character: {c}")),
        }
    }
    Ok(Expression::Sum(terms))
}

fn parse_term(iter: &mut CharIter) -> Result<Expression, String> {
    match iter.peek() {
        Some('(') => {
            iter.next();
            Ok(parse_expression(iter)?)
        }
        Some(_) => Ok(Expression::Value(parse_money(iter)?)),
        None => Err("unexpected end of string".to_owned()),
    }
}

fn parse_money(iter: &mut CharIter) -> Result<Money, String> {
    enum State {
        Idle,
        Euros(i64),
        Cents100(i64),
        Cents10(i64),
    }
    let mut state = State::Idle;
    while let Some(c) = iter.peek() {
        state = match state {
            State::Idle => State::Euros(parse_int(iter)?),
            State::Euros(e) => match c {
                ',' | '.' => {
                    iter.next();
                    State::Cents100(e * 100)
                }
                _ => return Ok(Money { cents: e * 100 }),
            },
            State::Cents100(cents) => {
                iter.next();
                State::Cents10(cents + 10 * to_digit(c)?)
            }
            State::Cents10(cents) => {
                let digit = match to_digit(c) {
                    Ok(d) => {
                        iter.next();
                        d
                    }
                    Err(_) => 0,
                };
                return Ok(Money {
                    cents: cents + digit,
                });
            }
        };
    }
    match state {
        State::Euros(e) => Ok(Money { cents: e * 100 }),
        State::Cents10(c) => Ok(Money { cents: c }),
        _ => Err("unexpected end of string".to_owned()),
    }
}

fn parse_int(iter: &mut CharIter) -> Result<i64, String> {
    enum State {
        Idle,
        Sign(i8),
        Int(i64),
    }
    let mut state = State::Idle;
    while let Some(c) = iter.next() {
        state = match state {
            State::Idle => match c {
                '+' => State::Sign(1),
                '-' => State::Sign(-1),
                _ => State::Int(to_digit(c)?),
            },
            State::Sign(s) => State::Int(s as i64 * to_digit(c)?),
            State::Int(i) => match to_digit(c) {
                Ok(d) => State::Int(i * 10 + d),
                _ => {
                    iter.go_back();
                    return Ok(i);
                }
            },
        };
    }
    match state {
        State::Int(i) => Ok(i),
        _ => Err("unexpected end of string".to_owned()),
    }
}

struct CharIter {
    data: Vec<char>,
    index: usize,
}

impl CharIter {
    fn new(data: Vec<char>) -> Self {
        Self { data, index: 0 }
    }

    fn next(&mut self) -> Option<char> {
        let c = self.data.get(self.index);
        self.index += 1;
        c.copied()
    }

    fn peek(&self) -> Option<char> {
        self.data.get(self.index).copied()
    }

    fn has_next(&self) -> bool {
        self.peek().is_some()
    }

    fn go_back(&mut self) -> &mut Self {
        self.index -= 1;
        self
    }
}

fn to_digit(c: char) -> Result<i64, String> {
    match c {
        '0'..='9' => Ok(c as i64 - '0' as i64),
        _ => Err(format!("invalid digit: {c}")),
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Value(money) => {
                if money.is_negative() {
                    write!(f, "-")?;
                }
                let euros = money.cents.abs() / 100;
                let cents = money.cents.abs() % 100;
                if cents == 0 {
                    write!(f, "{}", euros)
                } else {
                    write!(f, "{}.{:02}", euros, cents)
                }
            }
            Expression::Sum(terms) => {
                for (i, term) in terms.iter().enumerate() {
                    match term.sign {
                        Sign::Positive if i > 0 => write!(f, "+")?,
                        Sign::Negative => write!(f, "-")?,
                        _ => {}
                    }
                    write!(f, "{}", term.expression)?;
                }
                Ok(())
            }
            Expression::Multiplication(expr, i) | Expression::Division(expr, i) => {
                let use_parens = matches!(&**expr, Expression::Sum(_));
                if use_parens {
                    write!(f, "(")?;
                }
                write!(f, "{}", expr)?;
                if use_parens {
                    write!(f, ")")?;
                }
                write!(
                    f,
                    "{}",
                    if matches!(self, Expression::Multiplication(_, _)) {
                        '*'
                    } else {
                        '/'
                    }
                )?;
                write!(f, "{}", i)
            }
        }
    }
}
