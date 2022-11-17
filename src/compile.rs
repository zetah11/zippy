use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use cc::Build;
use target_lexicon::Triple;

use super::Arguments;

/// Emit some C code and compile it. Returns the path of the executable.
pub fn compile(args: &Arguments, target: &Triple, code: String) -> anyhow::Result<PathBuf> {
    let artifacts = Path::new("artifacts");

    DirBuilder::new().recursive(true).create(artifacts)?;

    let code_path = artifacts.join(args.path.with_extension("c"));

    {
        let mut file = File::create(&code_path)?;
        file.write_all(code.as_bytes())?;
    }

    let exec_name = args.path.with_extension("exe");
    let exec_name = exec_name.to_string_lossy();

    let mut build = Build::new();
    build
        .opt_level(0)
        .target(&target.to_string())
        .host(&Triple::host().to_string())
        .cargo_metadata(false)
        .warnings(false);

    let tool = build.get_compiler();
    let _output = tool
        .to_command()
        .current_dir(artifacts)
        .arg(code_path.strip_prefix(artifacts).unwrap())
        .output()?;

    Ok(PathBuf::from(exec_name.as_ref()))
}
