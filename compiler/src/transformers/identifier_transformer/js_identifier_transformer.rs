use crate::targets::Program;
use std::path::PathBuf;

pub struct JsIdentifierTransformer {}

impl JsIdentifierTransformer {
    pub fn new(_path: PathBuf) -> Self {
        Self {}
    }

    pub fn transform(&mut self, _program: &mut Program) {}
}
