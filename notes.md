# SLFinance

### Continue
- Display / export graphs
    - ascii
    - Which format? (pdf, png, svg, html, ascii art)
        - svg: probably best, since I an drawing the graph as individual shapes
          anyways
        - html: would probably just be a wrapper around the svg?
        - pdf: also probably a wrapper around the svg
        - png: since text will be involved (and i am not writing a text renderer
          for this project), it would probably just be a render of the svg
        - ascii art: perhaps as an alternative to svg. would fit the style since
          it is a cli tool

### Features to add
- slf edit
    - Specify an existing entry just like in slf remove (by index, default month
      is the current month, default list is expenses)
    - syntax: using already existing arguments, e.g. `slf edit 1 -d "new
    description"`
    - things to edit:
        - set amount
        - set / unset category
        - set / unset description
        - set / unset date
            - need to move entry to the corresponding list if month / year
            changes
        - set year / month
            - need to move entry, need to remove date
            - is this really so important? when would this be useful?
- Recurring incomes / expenses (rent, electricity, phone, etc.)
    - specify a json file (similar to tracker) of incomes and expenses (and i
      guess totals too), each with a description and category (if neccessary) to
      be inserted with a command like `slf import <filename>`
- slf list
    - For detailed display: if only one list type is given, show sums for
    individual categories in addition to final sum?
    - Options to filter displayed entries, e.g. by category or date?
- slf new
    - create a new tracker
- slf edit-category
    - Change a category's name
- Encrypted save files?
    - For now, I will probably just use age externally. Maybe this could be
    integrated into slfinance in the future. But it would perhaps just end up
    being a wrapper around age, which wouldn't really be a benefit.
    - Actually, add slf push and slf pull as wrappers to call the scripts at
    finances/(push|pull).sh?

### Problems to fix
- SerialTracker is 99% identical to Tracker. The only differences are that
  YearMonth is flattened and that fields have shorter names ("cat" instead of
  "category" etc.).
- Check that given aruments actually match the command

## Frontend
???
