#![allow(dead_code)]
pub use std::error::Error;
use std::fmt::Debug;
use std::path::Path;

pub type Result<T> = core::result::Result<T, Box<dyn Error>>;

pub fn get_parent_path(path: &str) -> Option<&str> {
    let path = std::path::Path::new(path);
    path.parent().map(|p| p.to_str()).flatten()
}

pub fn new_ref<T>(value: T) -> std::rc::Rc<std::cell::RefCell<T>> {
    std::rc::Rc::new(std::cell::RefCell::new(value))
}

pub fn read_file<P: AsRef<Path>>(path: P) -> String where P: Debug {
    std::fs::read_to_string(&path).expect(format!("Failed to read file content: {:?}", path).as_str())
}