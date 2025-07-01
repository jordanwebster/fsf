use super::{walk_ast, AstVisitor};
use crate::item::Item;
use crate::parse_module;
use crate::targets::{Module, Program};
use itertools::Itertools;
use std::path::PathBuf;

const TEST_RUNNER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/templates/test_runner.fsf"
));

pub struct TestRunnerTransformer {
    tests: Vec<(String, String)>,
    current_module: Option<PathBuf>,
    root: PathBuf,
}

impl TestRunnerTransformer {
    pub fn new(root: PathBuf) -> Self {
        Self {
            tests: Vec::new(),
            current_module: None,
            root,
        }
    }

    pub fn transform(&mut self, program: &mut Program) {
        walk_ast(program, self);

        let imports = self
            .tests
            .iter()
            .map(|(module, test)| format!("import {}::{};", module, test))
            .join("\n");

        let tests = self
            .tests
            .iter()
            .map(|(module, test)| format!("__RUN_TEST(\"{}::{}\", {});", module, test, test))
            .join("\n");

        let contents = TEST_RUNNER_TEMPLATE
            .replace("/* replace_imports */", &imports)
            .replace("/* replace_tests */", &tests);

        let test_runner = parse_module(contents, self.root.join("main.fsf")).unwrap();
        program.push(test_runner);

        // TODO: Set up pipelining this transformer with the name transformer before compilation
        // TODO: Implement __RUN_TEST as a special token that handles catching panics and reporting
        // test errors
    }
}

impl AstVisitor for TestRunnerTransformer {
    fn visit_module(&mut self, module: &mut Module) {
        self.current_module = Some(module.path.clone());
    }

    fn visit_item(&mut self, item: &mut Item) {
        match item {
            Item::Function { name, .. } if name.starts_with("test_") => {
                self.tests.push((
                    self.current_module
                        .as_ref()
                        .unwrap()
                        .strip_prefix(&self.root)
                        .unwrap()
                        .with_extension("")
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .join("::"),
                    name.clone(),
                ));
            }
            _ => (),
        }
    }
}
