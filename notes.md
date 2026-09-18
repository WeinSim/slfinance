# SLFinance

### Continue
- Properly implement --date and --month arguments

### Features to add
- Add incomes / expenses
    - How to specify categories?
        - Should categories be mandatory?
        - Specify by index / identifier, which is separate from the display name?
- Conversion
    - Option to convert back from .json to .tsv (or .csv) to import data back in excel
- List individual expenses
- Delete entries
- Encrypted save files?
- Export graphs
    - Which file format? (pdf, png, svg, html, ascii art)

### Problems to fix
- SerialTracker is 99% identical to Tracker. I think the only difference is that YearMonth
is flattened.
- Check that given aruments actually match the command

## Frontend
???



Parsing grammar:
expression = money | '(' expression ')' | sum | multiplication | division
sum = {expression ('+' | '-')} expression
multiplication = expression '*' int
division = expression '/' int

money = int [('.' | ',') digit digit]
<!-- int = ['+' | '-'] {digit} digit -->
int = {digit} digit
digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
