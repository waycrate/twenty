mod config;
mod daemon;
mod session_lock;

use std::{cmp, env, process::exit};

pub const RESET: &str = "\x1b[0m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const BOLD: &str = "\x1b[1m";
pub const UNDERLINE: &str = "\x1b[4m";

#[macro_export]
macro_rules! twenty_log {
    ($($arg:tt)*) => {
        println!("\x1b[32mTwenty:\x1b[0m {}", format!($($arg)*))
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len().cmp(&1) {
        cmp::Ordering::Equal => {
            help();
        }
        cmp::Ordering::Greater => match args[1].as_str() {
            "-h" | "--help" => {
                help();
            }
            "-i" | "--init" => {
                let mut cfg = config::load();

                if args.len() == 3 {
                    match args[2].as_str() {
                        "light" => cfg.theme = "light".to_string(),
                        "dark" => cfg.theme = "dark".to_string(),
                        _ => {
                            twenty_log!("Not a valid theme!");
                            exit(1);
                        }
                    }
                }
                daemon::start(cfg);
            }
            "-k" | "--kill" => {
                daemon::kill();
            }
            "-p" | "--pause" => {
                daemon::toggle_pause();
            }
            "-s" | "--status" => {
                daemon::status();
            }
            _ => {
                twenty_log!("Invalid option '{}'.", args[1]);
                exit(1);
            }
        },
        cmp::Ordering::Less => {
            exit(1);
        }
    }
}

fn help() {
    let help_msg = format!(
        "{GREEN}{BOLD}twenty {RESET} {version}
    Twenty makes sure that you look 20 ft away every 20 minutes for 20 seconds.

{YELLOW}USAGE:{RESET}
    twenty {GREEN}[OPTIONS]{RESET}

{YELLOW}OPTIONS:{RESET}
    {GREEN}-h, --help{RESET}
        Show this help message.
    {GREEN}-i, --init [dark/light]{RESET}
        Initialize the program. Defaults to dark theme.
    {GREEN}-k, --kill{RESET}
        Kill the program.
    {GREEN}-p, --pause{RESET}
        Pause the program.
    {GREEN}-s, --status{RESET}
        Show the status of the program (running / paused / not running).

Link: {UNDERLINE}{BLUE}https://github.com/waycrate/twenty{RESET}",
        version = env!("CARGO_PKG_VERSION"),
    );
    println!("{}", help_msg);
}
