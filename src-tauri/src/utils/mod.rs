mod core;
mod xml;
mod nodes;
mod wsdl;

pub use core::*;
use tauri::{Monitor, Window};

pub fn get_current_monitor(w: &Window) -> Result<Monitor> {
    let current_monitor = w.current_monitor()?.unwrap();
    Ok(current_monitor)
}

pub fn get_current_monitor_name(w: &Window) -> Result<String> {
    Ok(get_current_monitor(w)?.name().unwrap().to_string())
}