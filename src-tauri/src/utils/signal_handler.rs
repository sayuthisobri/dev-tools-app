use signal_hook::{consts::SIGSEGV, iterator::Signals};
use std::thread;
use tracing::{error, info};

pub fn setup_signal_handlers() {
    let mut signals = Signals::new(&[SIGSEGV]).expect("Failed to create signal handler");

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGSEGV => {
                    error!("Segmentation fault detected - likely unsafe code crash");
                    // Log additional context if available
                    log_crash_context();
                    // In production, you might want to restart or notify user
                    std::process::exit(1);
                }
                _ => {
                    info!("Received signal: {:?}", sig);
                }
            }
        }
    });
}

fn log_crash_context() {
    // Log current state, memory usage, etc.
    if let Ok(mem_info) = sys_info::mem_info() {
        error!("Memory info at crash: total={}, avail={}", mem_info.total, mem_info.avail);
    }
}