use std::{
    fs,
    io::{self, Write},
    sync::LazyLock,
};

mod arguments;
use chrono::Month;

use crate::{
    arguments::{ArgList, Argument},
    money::Tracker,
};

mod money;

static CONFIG: LazyLock<Config> = LazyLock::new(init_config);

const MONTHS: &[Month] = &[
    Month::January,
    Month::February,
    Month::March,
    Month::April,
    Month::May,
    Month::June,
    Month::July,
    Month::August,
    Month::September,
    Month::October,
    Month::November,
    Month::December,
];

fn main() {
    // parse arguments
    // skip first argument ('slfinance')
    let args = &std::env::args().collect::<Vec<String>>()[1..];
    let mut app_state: AppState = AppState::new();
    if process_input(args, &mut app_state) {
        run_interactive(&mut app_state);
    }
}

/// Returns whether the program should run in interactive mode if this is the initial call to slfinance.
fn process_input(input: &[String], app_state: &mut AppState) -> bool {
    let arg_list = match ArgList::parse(input) {
        Some(a) => a,
        None => {
            print_help();
            return false;
        }
    };
    // process arguments
    let mut run_interactive = true;
    let mut filename: Option<&str> = None;
    for arg in arg_list.args() {
        match arg {
            Argument::Help => {
                print_help();
                run_interactive = false;
            }
            Argument::Version => {
                print_version();
                run_interactive = false;
            }
            Argument::File(f) => {
                filename = Some(f);
            }
            Argument::Command(_) => {
                // should never happen
                panic!("Error: detected command in non-command arg list");
            }
        }
    }
    if let Some(t) = filename.and_then(load_file) {
        app_state.tracker = Some(t);
    }
    if let Some(command) = arg_list.command() {
        command.run(arg_list.args(), app_state);
        run_interactive = false;
    }
    run_interactive
}

fn run_interactive(app_state: &mut AppState) {
    println!("{}", CONFIG.version_message.lines().next().unwrap_or(""));
    println!("Type `help` for further information.");
    loop {
        print!("$ ");
        match io::stdout().flush() {
            Ok(()) => {}
            Err(e) => {
                println!("{e}");
            }
        }
        let mut user_input = String::new();
        match io::stdin().read_line(&mut user_input) {
            Ok(_) => {
                process_input(
                    &user_input
                        .split_whitespace()
                        .map(|s| s.to_owned())
                        .collect::<Vec<String>>(),
                    app_state,
                );
            }
            Err(e) => {
                println!("{e}");
            }
        }
    }
}

fn print_help() {
    print!("{}", CONFIG.help_message);
}

fn print_version() {
    print!("{}", CONFIG.version_message);
}

struct Config {
    help_message: String,
    version_message: String,
}

fn init_config() -> Config {
    Config {
        help_message: fs::read_to_string("res/help.txt")
            .unwrap_or("[Unable to load help message]".to_owned()),
        version_message: fs::read_to_string("res/version.txt")
            .unwrap_or("[Unable to load version message]".to_owned()),
    }
}

fn load_file(filename: &str) -> Option<Tracker> {
    let json = match fs::read_to_string(filename) {
        Ok(json) => json,
        Err(e) => {
            println!("{e}");
            return None;
        }
    };
    match serde_json::from_str(&json) {
        Ok(tracker) => tracker,
        Err(e) => {
            println!("{e}");
            None
        }
    }
}

struct AppState {
    tracker: Option<Tracker>,
}

impl AppState {
    fn new() -> Self {
        Self { tracker: None }
    }
}
