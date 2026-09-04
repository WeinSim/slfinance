use std::{env, fs, process::ExitCode, sync::LazyLock};

mod arguments;
mod money;
mod serial;
mod settings;
mod sutil;

use crate::{
    arguments::{ArgList, Argument},
    settings::Settings,
};

const DEV_BUILD: bool = true;
// TODO continue: use std::env::home_dir() to get the home directory
const SETTINGS_FILE: &str = "/home/simon/.slfinance/settings.json";

static CONFIG: LazyLock<Config> = LazyLock::new(init_config);
static SETTINGS: LazyLock<Settings> = LazyLock::new(load_settings);

// const MONTHS: &[Month] = &[
//     Month::January,
//     Month::February,
//     Month::March,
//     Month::April,
//     Month::May,
//     Month::June,
//     Month::July,
//     Month::August,
//     Month::September,
//     Month::October,
//     Month::November,
//     Month::December,
// ];

fn main() -> ExitCode {
    if DEV_BUILD {
        sutil::print_num_lines();
    }
    let args = &std::env::args().collect::<Vec<String>>()[1..];
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            println!("{}", msg);
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let arg_list = ArgList::parse(args).ok_or(CONFIG.help_message.clone())?;
    // process arguments
    let mut filename: Option<String> = None;
    let mut run_command = true;
    for arg in arg_list.args() {
        match arg {
            Argument::Help => {
                print_help();
                run_command = false;
            }
            Argument::Version => {
                print_version();
                run_command = false;
            }
            Argument::File(f) => {
                filename = Some(f.to_owned());
            }
            Argument::Command(_) => {
                // should never happen
                return Err("Error: detected command in non-command arg list".to_owned());
            }
            _ => {}
        }
    }
    if !run_command {
        return Ok(());
    }
    let mut tracker = serial::load_file(
        &filename.unwrap_or(
            SETTINGS
                .get("lastOpenedFile".to_owned())
                .ok_or("Unable to find last opened file. Use --file to specify a file to open")?,
        ),
    )?;
    // format!("Unable to open file \"{filename}\": {msg}");
    // }
    if let Some(command) = arg_list.command() {
        command.run(arg_list.args(), &mut tracker)
    } else {
        Err(CONFIG.help_message.clone())
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

fn load_settings() -> Settings {
    match Settings::load(SETTINGS_FILE) {
        Ok(settings) => settings,
        Err(e) => {
            println!("Unable to load settings: {e}");
            Settings::default()
        }
    }
}
