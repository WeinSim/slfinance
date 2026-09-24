# SLFinance

### Continue

### Features to add
- slf convert
    - Option to convert back from .json to .tsv (or .csv) to import data back in excel
- Check that given aruments actually match the command
- slf list
    - For detailed display: if only one list type is given, show sums for individual
    categories in addition to final sum?
        - This would require everything to be sorted by category first, not by date first
    - Options to filter displayed entries, e.g. by category or date?
- Display / export graphs
    - Which format? (pdf, png, svg, html, ascii art)
- Encrypted save files?
    - For now, I will probably just use age externally. Maybe this could be integrated
    into slfinance in the future. But it would perhaps just end up being a wrapper around
    age, which wouldn't really be a benefit.
    - Actually, add slf push and slf pull as wrappers to call the scripts at
    finances/(push|pull).sh?

### Problems to fix
- Sort MoneyChanges by date (in the data structure itself)
- SerialTracker is 99% identical to Tracker. I think the only difference is that YearMonth
is flattened.

## Frontend
???
