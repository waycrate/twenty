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
    Running,
    Paused,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            State::Running => "running",
            State::Paused => "paused",
        })
    }
}

fn read_state() -> State {
    match fs::read_to_string(state_path()).map(|s| s.trim().to_string()) {
        Ok(s) if s == "paused" => State::Paused,
        _ => State::Running,
    }
}

fn write_state(state: State) {
    let _ = fs::write(state_path(), state.to_string());
}

// --status
pub fn status() {
    if is_running() {
        twenty_log!("Currently {}.", read_state());
    } else {
        twenty_log!("Not running.");
    }
}

// --pause
pub fn toggle_pause() {
    if !is_running() {
        twenty_log!("Not running.");
        return;
    }
    let next = match read_state() {
        State::Paused => State::Running,
        State::Running => State::Paused,
    };
    write_state(next);
    twenty_log!("Now {}.", next);

    Notification::new()
        .summary("Twenty: toggled status")
        .body(&format!("Twenty is now {}.", next))
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

    write_state(State::Running);
    run_loop(cfg);
}

fn run_loop(cfg: Config) {
    loop {
        countdown(cfg.cooldown);

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
        thread::sleep(Duration::from_secs(10));
        if blacklisted_running(&cfg.blacklisted) || read_state() == State::Paused {
            continue;
        }

        let _ = session_lock::lock(cfg.theme == "dark", cfg.lock_timer.as_secs());
    }
}

fn countdown(secs: Duration) {
    // avoid integer underflow; leave 10s for the notification warning
    let mut remaining = secs.as_secs().saturating_sub(10);
    while remaining > 0 {
        thread::sleep(Duration::from_secs(1));
        if read_state() == State::Paused {
            continue;
        }
        remaining -= 1;
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
