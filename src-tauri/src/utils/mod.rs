#![allow(dead_code)]

mod core;
mod nodes;
pub(crate) mod progress_helper;
mod wsdl;
mod xml;


pub use core::*;
use std::path::{Path, PathBuf};
use tauri::{Monitor, Window};

pub fn get_current_monitor(w: &Window) -> Result<Monitor> {
    let current_monitor = w.current_monitor()?.unwrap();
    Ok(current_monitor)
}

pub fn get_current_monitor_name(w: &Window) -> Result<String> {
    Ok(get_current_monitor(w)?.name().unwrap().to_string())
}

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    if let Some(str_path) = p.to_str() {
        if str_path == "~" || str_path.starts_with("~/") {
            if let Some(home) = dirs_next::home_dir() {
                // remove the leading "~" or "~/" and join the rest
                let mut rest = &str_path[1..]; // e.g., "~/foo" -> "/foo" or "~" -> ""
                if rest.starts_with('/') {
                    rest = &rest[1..];
                }
                return if rest.is_empty() {
                    home
                } else {
                    home.join(rest)
                };
            }
        }
    }
    p.to_path_buf()
}


#[cfg(test)]
pub(crate) mod test {
    use log::LevelFilter;
    pub fn init_test_logger(level_filter: LevelFilter) {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(level_filter)
            .try_init();
    }
}