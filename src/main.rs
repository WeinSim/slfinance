use std::{
    fs,
    path::PathBuf,
    process::ExitCode,
    sync::{LazyLock, RwLock},
};

use dirs;

mod arguments;
mod csv_conversion;
mod money;
mod serial;
mod settings;
mod sutil;

use crate::{
    arguments::{ArgList, Argument},
    csv_conversion::convert,
    settings::Settings,
};

const DEV_BUILD: bool = false;

static CONFIG: LazyLock<Config> = LazyLock::new(init_config);
static SETTINGS: LazyLock<RwLock<Settings>> = LazyLock::new(load_settings);
static SETTINGS_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(get_settings_path);

fn main() -> ExitCode {
    if DEV_BUILD {
        sutil::print_num_lines();
    }
    let args = &std::env::args().collect::<Vec<String>>()[1..];
    // match run(args) {
    match convert(args) {
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
            _ => {}
        }
    }
    if !run_command {
        return Ok(());
    }
    let filename = &filename.unwrap_or(
        SETTINGS
            .read()
            .map_err(|_| "Settings lock poisoned")?
            .get("lastOpenedFile")
            .ok_or("Unable to find last opened file. Use --file to specify a file to open")?,
    );
    SETTINGS
        .write()
        .map_err(|_| "Settings lock poisoned")?
        .set("lastOpenedFile".to_string(), filename)
        .map_err(|e| e.to_string())?;
    let mut tracker = serial::load_file(filename)?;
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
