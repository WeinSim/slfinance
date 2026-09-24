use std::{
    fs,
    path::PathBuf,
    process::ExitCode,
    sync::{LazyLock, RwLock},
};

use chrono::{Local, Month, NaiveDate};

mod commands;
mod expressions;
mod money;
mod serial;
mod settings;
mod sutil;

use crate::{commands::Arguments, settings::Settings};

const DEV_BUILD: bool = false;

const MONTHS: [Month; 12] = [
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

static CONFIG: Config = Config {
    help_message: include_str!("resources/help.txt"),
    version_message: include_str!("resources/version.txt"),
};
static SETTINGS: LazyLock<RwLock<Settings>> = LazyLock::new(load_settings);
static SETTINGS_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(get_settings_path);
static TODAY: LazyLock<NaiveDate> = LazyLock::new(|| Local::now().date_naive());

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

fn run(args_raw: &[String]) -> Result<(), String> {
    // let args = Arguments {
    //     command: Add(1),
    //     description: Some("test".to_owned()),
    //     ..Default::default()
    // };
    let args = Arguments::parse(args_raw)?;
    // process arguments
    let mut run_command = true;
    if args.help.is_some_and(|b| b) {
        print_help();
        run_command = false;
    }
    if args.version.is_some_and(|b| b) {
        print_version();
        run_command = false;
    }
    if !run_command {
        return Ok(());
    }
    if let Some(ref command) = args.command {
        command.run(&args)
    } else {
        Err(CONFIG.help_message.to_owned())
    }
}

fn print_help() {
    print!("{}", CONFIG.help_message);
}

fn print_version() {
    print!("{}", CONFIG.version_message);
}

struct Config {
    help_message: &'static str,
    version_message: &'static str,
}

fn get_settings_path() -> Option<PathBuf> {
    let dir = dirs::home_dir()?.join(".slfinance");
    match fs::create_dir_all(&dir) {
        Ok(()) => {}
        Err(_) => {
            println!("Unable to create ~/.slfinance directory");
        }
    }
    Some(dir.join("settings.json"))
}

fn load_settings() -> RwLock<Settings> {
    fn inner() -> Option<Settings> {
        match Settings::load(SETTINGS_PATH.as_ref()?) {
            Ok(settings) => Some(settings),
            Err(e) => {
                println!("Unable to parse settings file: {e}");
                None
            }
        }
    }
    RwLock::new(inner().unwrap_or_default())
}
