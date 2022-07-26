use std::fs::read_to_string;
use std::fs::create_dir;
use std::fmt::Display;
use std::process::exit;
use std::process::ExitStatus;
use std::path::Path;
use std::io::Error;
use std::io::ErrorKind;

use toml::Value;

use pico_args::Arguments;

mod build_cells;
mod link_nano_core;
mod serialize_nano_core_syms;

pub fn die() -> ! {
    exit(1)
}

fn main() {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        // println!("{}", include_str!("help.txt"));
        println!("sorry, no help atm");
    } else {
        let config_path = match args.value_from_str(["-c", "--config-file"]) {
            Ok(path) => path,
            _ => "config.toml".to_string(),
        };

        log!("reading config", "config file: {}", config_path);

        let cfg_string = match read_to_string(&config_path) {
            Ok(cfg_string) => cfg_string,
            Err(e) => oops!("reading config", "{}", e),
        };

        let config = match cfg_string.parse::<Value>() {
            Ok(cfg_string) => cfg_string,
            Err(e) => oops!("parsing config", "{}", e),
        };

        let steps = [
            build_cells::process,
            link_nano_core::process,
            serialize_nano_core_syms::process,
        ];

        log!("parsing config", "configuration was parsed successfully");

        for step in steps {
            step(&config);
        }
    }
}

#[macro_export]
macro_rules! log {
    ($log_stage:expr, $($arg:tt)*) => {{
        print!("[{}] ", $log_stage);
        println!($($arg)*);
    }}
}

#[macro_export]
macro_rules! oops {
    ($log_stage:expr, $($arg:tt)*) => {{
        print!("[{}] error: ", $log_stage);
        println!($($arg)*);
        crate::die();
    }}
}

fn check_result(stage: &str, result: Result<ExitStatus, Error>, errmsg: &str) {
    let no_problem = match result {
        Ok(result) => result.success(),
        _ => false,
    };

    if !no_problem {
        oops!(stage, "{}", errmsg);
    }
}

fn try_create_dir<P: AsRef<Path> + Display>(path: P) {
    if let Err(e) = create_dir(&path) {
        if e.kind() != ErrorKind::AlreadyExists {
            println!("could not create directory: {}", path);
            crate::die();
        }
    }
}

fn opt_default(path: &[&str]) -> Value {
    let mut config = &include_str!("defaults.toml").parse::<Value>().unwrap();
    for key in path {
        if let Some(value) = config.get(key) {
            config = value;
        } else {
            println!("missing option in config: {}", path.join("/"));
            crate::die();
        }
    }
    config.clone()
}

pub fn opt(mut config: &Value, path: &[&str]) -> Value {
    for key in path {
        if let Some(value) = config.get(key) {
            config = value;
        } else {
            return opt_default(path);
        }
    }
    config.clone()
}

pub fn opt_str(config: &Value, path: &[&str]) -> String {
    if let Value::String(string) = opt(config, path) {
        string
    } else {
        println!("wrong type: {} must be a string!", path.last().unwrap());
        crate::die();
    }
}
