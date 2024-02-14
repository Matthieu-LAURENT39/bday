![Crates.io](https://img.shields.io/crates/v/bday)
[![Build](https://github.com/Matthieu-LAURENT39/bday/actions/workflows/build.yaml/badge.svg)](https://github.com/Matthieu-LAURENT39/bday/actions/workflows/build.yaml)
[![Test](https://github.com/Matthieu-LAURENT39/bday/actions/workflows/test.yaml/badge.svg)](https://github.com/Matthieu-LAURENT39/bday/actions/workflows/test.yaml)

# 🎂 Bday 🎉

Easily remember and see upcoming birthdays 🎂


## Features
- Show how far away birthdays are
- Support for dates without specifying the year
- Support for timezones, so you can wish your friends a happy birthday when the clock hits midnight in their country
- Blazingly fast, even with large datasets ⚡️


## Usage
```bash
# Adds a birthday
$ bday add --name "Hiyajo Maho" --date 02/11/1989
Added entry for Hiyajo Maho, born: 02/11/1989

# Adds a birthday, without specifying the year
$ bday add --name "Akiha Rumiho" --date 03/04
Added entry for Akiha Rumiho, born: 03/04

# List all birthdays
$ bday list
╭───┬──────────────┬─────────────┬─────────┬─────────────╮
│ # │ Name         │ Date        │ Age     │ In          │
├───┼──────────────┼─────────────┼─────────┼─────────────┤
│ 1 │ Hiyajo Maho  │ 02 November │ 34 🡒 35 │ in 8 months │
├───┼──────────────┼─────────────┼─────────┼─────────────┤
│ 2 │ Akiha Rumiho │ 03 April    │ ?       │ in 2 months │
╰───┴──────────────┴─────────────┴─────────┴─────────────╯

# List birthdays that will occur before a date
$ bday list --before 15/05/2024
╭───┬──────────────┬──────────┬─────┬─────────────╮
│ # │ Name         │ Date     │ Age │ In          │
├───┼──────────────┼──────────┼─────┼─────────────┤
│ 1 │ Akiha Rumiho │ 03 April │ ?   │ in 2 months │
╰───┴──────────────┴──────────┴─────┴─────────────╯

# List only the n birthdays closest to today
$ bday list --limit 1
╭───┬──────────────┬──────────┬─────┬─────────────╮
│ # │ Name         │ Date     │ Age │ In          │
├───┼──────────────┼──────────┼─────┼─────────────┤
│ 1 │ Akiha Rumiho │ 03 April │ ?   │ in 2 months │
╰───┴──────────────┴──────────┴─────┴─────────────╯
```

**Display the birthdays that will happen in the next 7 days**  
A personal favorite of mine is adding this to my `~/.bashrc` to display upcoming birthdays.  
```bash
bday -f /tmp/aaa.toml list --before $(date -d "+7 days" "+%Y-%m-%d") 2> /dev/null
```
Note that this displays nothing if no birthdays are coming up.

## Installation
### With Cargo
```bash
cargo install bday
```

### From source
```bash
git clone "https://github.com/Matthieu-LAURENT39/bday"
cd bday
cargo install --path .
```


## Special thanks
This project was inspired by [IonicaBizau's "birthday" tool](https://github.com/IonicaBizau/birthday).  
I wanted to try making my own version in Rust as a learning experience.


## License
This program is free software; you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation; either version 2 of the License, or (at your option) any later version.