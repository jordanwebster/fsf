use crate::compilers::Program;
use anyhow::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct GoCompiler {}

impl GoCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&mut self, program: Program, compile_dir: &Path) -> Result<()> {
        for module in program {
            let output = module
                .items
                .into_iter()
                .filter_map(|stmt| stmt.map(|s| s.compile()))
                .join("");
            let mut output_path = compile_dir.join(module.path.file_stem().unwrap());
            output_path.set_extension("go");
            let mut output_file = File::create(&output_path)?;
            output_file.write_all("package main\n".as_bytes())?;
            // TODO: Propagate this information up via the parser
            if output.contains("fmt.Println") || output.contains("fmt.Sprintf") {
                output_file.write_all("import \"fmt\"\n".as_bytes())?;
            }
            output_file.write_all(output.as_bytes())?;
        }

        Ok(())
    }
}
