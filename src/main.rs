mod session_lock;
use notify_rust::Notification;
use std::{cmp, env, process::exit, sync::atomic, thread, time};

#[macro_export]
macro_rules! twenty_log {
    ($($arg:tt)*) => {
        println!("\x1b[32mTwenty:\x1b[0m {}", format!($($arg)*));
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
            "-k" | "--kill" => {
                kill_twenty();
            }
            "-i" | "--init" => {
                if args.len() == 3 {
                    match args[2].as_str() {
                        "light" => {
                            session_lock::IS_DARKMODE.store(false, atomic::Ordering::Relaxed);
                        }
                        "dark" => {
                            session_lock::IS_DARKMODE.store(true, atomic::Ordering::Relaxed);
                        }
                        _ => {
                            twenty_log!("Not a valid theme!");
                            exit(1);
                        }
                    }
                }
                init();
            }
            "-l" | "--lightmode" => {
                init();
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

fn init() {
    twenty_log!("[Re]started twenty. Screen will be locked in 20 minutes for 20 seconds.");
    let twenty_mins_minus_ten_secs = time::Duration::from_secs(1190);
    thread::sleep(twenty_mins_minus_ten_secs);

    Notification::new()
        .summary("10 seconds remaining before lock")
        .body("Your screen will get locked for 20 seconds to make sure that you relax your eyes. Run twenty -k to stop.")
        .show()
        .unwrap();

    let ten_secs = time::Duration::from_secs(10);
    thread::sleep(ten_secs);
    let _ = session_lock::lock();

    loop {
        if session_lock::UNLOCKED.load(atomic::Ordering::Relaxed) {
            init();
        }
    }
}

fn kill_twenty() {
    #[cfg(unix)]
    {
        std::process::Command::new("pkill")
            .arg("twenty")
            .output()
            .ok();
    }
}

fn help() {
    let help_msg = format!(
        "\x1b[32m\x1b[1mTwenty \x1b[0m {}
    Twenty makes sure that you look 20 ft away every 20 minutes for 20 seconds.

\x1b[33mUSAGE:\x1b[0m
    twenty \x1b[32m[OPTIONS]\x1b[0m

\x1b[33mOptions:\x1b[0m
    \x1b[32m-h, --help\x1b[0m
        Show this help message.
    \x1b[32m-i, --init [dark/light]\x1b[0m
        Initialize the program. Defaults to dark theme.
    \x1b[32m-k, --kill\x1b[0m
        Kill the program.
       
Link: \x1b[4m\x1b[34mhttps://github.com/rv178/twenty\x1b[0m",
        env!("CARGO_PKG_VERSION")
    );
    println!("{}", help_msg);
}
