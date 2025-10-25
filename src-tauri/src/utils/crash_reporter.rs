use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static CRASH_LOG_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

pub fn init_crash_reporting(log_path: PathBuf) {
    *CRASH_LOG_PATH.lock().unwrap() = Some(log_path);
}

pub fn log_crash(message: &str, location: &str, line: u32) {
    let crash_entry = format!("[CRASH] {} at {}:{}\n", message, location, line);
    log::error!("{}", crash_entry);
    if let Some(log_path) = CRASH_LOG_PATH.lock().unwrap().as_ref() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = file.write_all(crash_entry.as_bytes());
        }
    }
}

// In the panic hook, call this function
pub fn report_panic(panic_info: &std::panic::PanicHookInfo) {
    let location = panic_info
        .location()
        .unwrap_or_else(|| std::panic::Location::caller());
    let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        s
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s
    } else {
        "Unknown panic"
    };

    log_crash(message, location.file(), location.line());
}
