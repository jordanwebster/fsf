use crate::item::Item;
use std::path::PathBuf;

pub struct Module {
    pub path: PathBuf,
    pub items: Vec<Option<Item>>,
}

pub type Program = Vec<Module>;
pub mod go_compiler;
pub mod js_compiler;
