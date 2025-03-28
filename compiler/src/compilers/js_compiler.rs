use crate::compilers::Program;
use anyhow::Result;
use std::path::PathBuf;

pub struct JsCompiler {}

impl JsCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&mut self, program: Program) -> Result<PathBuf> {
        todo!();
    }
}
