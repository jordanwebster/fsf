use crate::compilers::Program;
use crate::item::Item;

pub struct TestCollector {}

impl TestCollector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn all_tests(program: &Program) -> Vec<String> {
        program
            .iter()
            .flat_map(|module| {
                module.items.iter().filter_map(|item| match item {
                    Item::Function { name, .. } if name.starts_with("test") => Some(name),
                    _ => None,
                })
            })
            .cloned()
            .collect()
    }
}
