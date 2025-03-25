use crate::item::Item;

pub struct TestCollector {}

impl TestCollector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn all_tests(items: &Vec<Option<Item>>) -> Vec<String> {
        items
            .iter()
            .filter_map(|item| match item {
                Some(Item::Function { name, .. }) => Some(name),
                _ => None,
            })
            .cloned()
            .collect()
    }
}
