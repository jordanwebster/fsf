use crate::parse_module;
use crate::targets::Program;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct StandardLibraryTransformer {
    root: PathBuf,
}

impl StandardLibraryTransformer {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn transform(&mut self, program: &mut Program) -> anyhow::Result<()> {
        let std_lib_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../std"));
        let mut modules = vec![];

        for entry in WalkDir::new(std_lib_dir) {
            let entry = entry?;
            let path = entry.path();

            // Check if it's a file and has the desired extension
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("fsf") {
                let relative_path = path.strip_prefix(std_lib_dir)?;
                match std::fs::read_to_string(path) {
                    Ok(contents) => {
                        let module =
                            parse_module(contents, self.root.join("std").join(relative_path))?;
                        modules.push(module);
                    }
                    Err(e) => eprintln!("Error reading {}: {}", path.display(), e),
                }
            }
        }
        program.extend(modules);
        Ok(())
    }
}
