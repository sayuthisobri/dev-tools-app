mod core;
mod nodes;
mod wsdl;
mod xml;

pub use core::*;
use log::LevelFilter;
use tauri::{Monitor, Window};

pub fn get_current_monitor(w: &Window) -> Result<Monitor> {
    let current_monitor = w.current_monitor()?.unwrap();
    Ok(current_monitor)
}

pub fn get_current_monitor_name(w: &Window) -> Result<String> {
    Ok(get_current_monitor(w)?.name().unwrap().to_string())
}

pub fn init_test_logger(level_filter: LevelFilter) {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(level_filter)
        .try_init();
}
