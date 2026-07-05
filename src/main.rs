mod config;
mod daemon;
mod session_lock;

use std::{cmp, env, process::exit};

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
        "\x1b[32m\x1b[1mTwenty \x1b[0m {}
    Twenty makes sure that you look 20 ft away every 20 minutes for 20 seconds.

\x1b[33mUSAGE:\x1b[0m
    twenty \x1b[32m[OPTIONS]\x1b[0m

\x1b[33mOPTIONS:\x1b[0m
    \x1b[32m-h, --help\x1b[0m
        Show this help message.
    \x1b[32m-i, --init [dark/light]\x1b[0m
        Initialize the program. Defaults to dark theme.
    \x1b[32m-k, --kill\x1b[0m
        Kill the program.
    \x1b[32m-p, --pause\x1b[0m
        Pause the program.
    \x1b[32m-s, --status\x1b[0m
        Show the status of the program (running / paused / not running).
       
Link: \x1b[4m\x1b[34mhttps://github.com/waycrate/twenty\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!("{}", help_msg);
}
