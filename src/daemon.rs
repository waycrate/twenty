use std::{env, fs, path::PathBuf, process::Command, thread, time::Duration};

use daemonize::Daemonize;
use notify_rust::Notification;

use crate::{config::Config, session_lock, twenty_log};

fn runtime_dir() -> PathBuf {
    let base = env::var("XDG_RUNTIME_DIR").unwrap_or("/tmp".to_string());
    let dir = PathBuf::from(base).join("twenty");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn pid_path() -> PathBuf {
    runtime_dir().join("pid")
}

fn state_path() -> PathBuf {
    runtime_dir().join("state")
}

fn read_pid() -> Option<u32> {
    fs::read_to_string(pid_path())
        .ok()?
        .trim()
        .parse::<u32>()
        .ok()
}

pub fn is_running() -> bool {
    match read_pid() {
        Some(pid) => PathBuf::from(format!("/proc/{pid}")).exists(),
        None => false,
    }
}

#[derive(PartialEq, Clone, Copy)]
enum State {
    Running(u32),
    Paused,
}

impl State {
    fn label(&self) -> &str {
        match self {
            State::Running(_) => "running",
            State::Paused => "paused",
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::Running(time) => write!(f, "running {}", time),
            State::Paused => write!(f, "paused"),
        }
    }
}

fn read_state() -> State {
    let s = fs::read_to_string(state_path()).unwrap_or_default();
    let mut parts = s.split_whitespace();
    match parts.next() {
        Some("paused") => State::Paused,
        _ => State::Running(
            parts
                .next()
                .and_then(|time| time.parse::<u32>().ok())
                .unwrap_or(0),
        ),
    }
}

fn write_state(state: State) {
    let _ = fs::write(state_path(), state.to_string());
}

// --status
pub fn status() {
    if !is_running() {
        twenty_log!("Not running.");
        return;
    }

    match read_state() {
        State::Running(time) => {
            twenty_log!(
                "Currently running. [{}m {}s remaining]",
                time / 60,
                time % 60
            );
        }
        State::Paused => {
            twenty_log!("Currently paused.");
        }
    }
}

// --pause
pub fn toggle_pause() {
    if !is_running() {
        twenty_log!("Not running.");
        return;
    }
    let next = match read_state() {
        State::Paused => State::Running(0),
        State::Running(_) => State::Paused,
    };
    write_state(next);
    twenty_log!("Now {}.", next.label());

    Notification::new()
        .summary("Twenty: toggled status")
        .body(&format!("Twenty is now {}.", next.label()))
        .show()
        .unwrap();
}

// --kill
pub fn kill() {
    match read_pid() {
        Some(pid) => {
            Command::new("kill").arg(pid.to_string()).status().ok();
            let _ = fs::remove_file(pid_path());
            let _ = fs::remove_file(state_path());
            twenty_log!("Stopped.");
        }
        None => twenty_log!("Not running."),
    }
}

// --init
pub fn start(cfg: Config) {
    if is_running() {
        twenty_log!("Already running!");
        return;
    }

    twenty_log!(
        "Started. Screen will lock every {:?} for {:?}.",
        cfg.cooldown,
        cfg.lock_timer
    );

    let daemon = Daemonize::new().pid_file(pid_path());
    if let Err(e) = daemon.start() {
        twenty_log!("Failed to start daemon: {}.", e);
        return;
    }

    write_state(State::Running(0));
    run_loop(cfg);
}

fn run_loop(cfg: Config) {
    loop {
        // avoid integer underflow; leave 10s for the notification warning
        countdown(cfg.cooldown.as_secs().max(10) as u32, 10);

        if blacklisted_running(&cfg.blacklisted) {
            Notification::new()
                .summary("Twenty: blacklisted processes running")
                .body(&format!(
                    "Skipping lock. Blacklisted processes: {:?}",
                    &cfg.blacklisted
                ))
                .show()
                .unwrap();
        } else {
            Notification::new()
                .summary("Twenty: 10 seconds remaining before lock")
                .body("Look away soon. Run `twenty -k` to stop, `twenty -p` to pause.")
                .show()
                .unwrap();
        }

        // wait 10 seconds before locking screen
        countdown(10, 0);
        if blacklisted_running(&cfg.blacklisted) || read_state() == State::Paused {
            continue;
        }

        let _ = session_lock::lock(cfg.theme == "dark", cfg.lock_timer.as_secs());
    }
}

fn countdown(from: u32, to: u32) {
    let mut remaining = from;
    while remaining > to {
        thread::sleep(Duration::from_secs(1));
        if read_state() == State::Paused {
            continue;
        }
        remaining -= 1;
        write_state(State::Running(remaining));
    }
}

fn blacklisted_running(list: &[String]) -> bool {
    list.iter().any(|name| {
        Command::new("pgrep")
            .arg("-x")
            .arg(name)
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    })
}
