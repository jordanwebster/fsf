use crate::expression::ExpressionWithoutBlock;
use crate::item::Item;
use crate::targets::{Module, Program};
use crate::token::Literal;
use crate::transformers::ast_visitor::{walk_ast, AstVisitor};
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct JsIdentifierTransformer {
    name_map: HashMap<String, String>,
    current_module: Option<PathBuf>,
    root: PathBuf,
}

impl JsIdentifierTransformer {
    pub fn new(root: PathBuf) -> Self {
        Self {
            name_map: HashMap::new(),
            current_module: None,
            root,
        }
    }

    pub fn transform(&mut self, program: &mut Program) {
        walk_ast(program, self)
    }
}

impl AstVisitor for JsIdentifierTransformer {
    fn visit_module(&mut self, module: &mut Module) {
        self.current_module = Some(module.path.clone());
        self.name_map.clear();
    }

    fn visit_item(&mut self, item: &mut Item) {
        match item {
            Item::Function { name, .. } => {
                // TODO: Add transformer step to find main function and add bootstrapping
                if name == "main" {
                    return;
                }

                let new_name = format!(
                    "{}_{}",
                    self.current_module
                        .as_ref()
                        .unwrap()
                        .with_extension("")
                        .strip_prefix(&self.root)
                        .unwrap()
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .join("_"),
                    name
                );
                self.name_map.insert(name.clone(), new_name.clone());
                name.clear();
                name.push_str(&new_name);
            }
            Item::Import { path } => {
                let name = path.last().unwrap().to_string();
                let full_path = self
                    .current_module
                    .as_ref()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .strip_prefix(&self.root)
                    .iter()
                    .map(|p| p.to_string_lossy())
                    .filter(|p| !p.is_empty())
                    .chain(path.iter().map(std::borrow::Cow::from))
                    .join("_");
                self.name_map.insert(name, full_path);
            }
            _ => (),
        }
    }

    fn visit_expression_without_block(&mut self, expr: &mut ExpressionWithoutBlock) {
        if let ExpressionWithoutBlock::Variable(token) = expr {
            if let Some(new_name) = self.name_map.get(&token.lexeme) {
                token.lexeme = new_name.clone();
                token.value = Some(Literal::Identifier(new_name.clone()));
            }
        }
    }
}
