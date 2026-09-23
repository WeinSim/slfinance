# SLFinance

### Continue
- slf delete

### Features to add
- slf convert
    - Option to convert back from .json to .tsv (or .csv) to import data back in excel
- Check that given aruments actually match the command
- Export graphs
    - Which file format? (pdf, png, svg, html, ascii art)
- Encrypted save files?

### Problems to fix
- Sort MoneyChanges by date (in the data structure itself)
- SerialTracker is 99% identical to Tracker. I think the only difference is that YearMonth
is flattened.

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
