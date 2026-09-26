# SLFinance

SLFinance is a program for managing private finances.

*It is currently in very early development.* Note that parts of the
documentation, including this README and the output of `slfinance --help`, might
be incorrect.

## Basic structure

SLFinance operates on money **trackers**. A tracker has three money **lists**:
one for the total money at the beginning of every month ("total"), one for
incomes ("incomes") and one for expenses ("expenses"). A list can have a number
of **entries** for every month. An entry always has an amount of money
associated with it (which can be specified directly or as a simple
[formula](#formulas)). An entry can optionally have associated with it the
following things:
- a date (not possible in the "total" list, where each entry is assumed to be on
the first day of the corresponding month)
- a description
- a category (every list has separate categories)

```
Tracker
├── Total (list)
│   ├── Entries for Jan 26
│   │   ├── Cash: 100€
│   │   ├── Bank account: 500€
│   │   └── ...
│   ├── Entries for Feb 26
│   └── ...
├── Incomes (list)
│   ├── Entries for Jan 26
│   ├── Entries for Feb 26
│   └── ...
└── Expenses (list)
    ├── Entries for Jan 26
    ├── Entries for Feb 26
    └── ...
```

The tracker is stored as a JSON file, which should be quite intuitive to
understand. The `"category"` field is an index into the list's categories.

The most recently used tracker will be saved in the settings file, which is
located at `~/.slfinance/settings.json` and will be opened by default the next
time `slfinance` is run.

## Commands

For an explanation of all commands and arguments, run `slfinance --help` or view
the [help message](src/resources/help.txt) directly.

## Formulas

A formula can consist of:
- money literals: either an integer or a decimal number (with `.` as the decimal
point) and at most two decimal places
- additions / subtractions: `expression_1 + expression_2 - expression_3`.
Leading signs are also supported.
- multiplication / divisions from the right: `expression_1 * 3 + expression_2 /
  2`
- parentheses: `(expression_1 + expression_2) / 3`

The full parsing grammar is listed below. Note that not all "invalid" inputs
(w.r.t. this grammar) are rejected by the parser:

```
expression = money | '(' expression ')' | sum | multiplication | division
sum = {expression ('+' | '-')} expression
multiplication = expression '*' int
division = expression '/' int

money = int [('.' | ',') digit digit]
int = ['+' | '-'] {digit} digit
digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
```