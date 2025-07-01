use crate::item::Item;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Module {
    pub path: PathBuf,
    pub items: Vec<Item>,
}

pub type Program = Vec<Module>;
pub mod go_target;
pub mod js_target;
