use std::env;
use std::ffi::OsStr;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::anyhow;
use cc::Build;
use target_lexicon::Triple;

use super::Arguments;

/// Emit some C code and compile it. Returns the path of the executable.
pub fn compile(args: &Arguments, target: &Triple, code: String) -> anyhow::Result<PathBuf> {
    let project_dir = env::current_dir()?;
    let artifacts = args.options().artifacts.as_path();

    DirBuilder::new().recursive(true).create(artifacts)?;

    let path = &args.options().path;
    let code_path = artifacts.join(path.with_extension("c"));

    {
        let mut file = File::create(&code_path)?;
        file.write_all(code.as_bytes())?;
    }

    let mut exec_name = path.with_extension("");

    let mut build = Build::new();
    build
        .opt_level(0)
        .target(&target.to_string())
        .host(&Triple::host().to_string())
        .cargo_metadata(false)
        .warnings(false);

    let tool = build.get_compiler();

    let args: Vec<&OsStr> = if tool.is_like_clang() || tool.is_like_gnu() {
        vec![&OsStr::new("-o"), exec_name.as_os_str()]
    } else if tool.is_like_msvc() {
        exec_name.set_extension("exe");
        Vec::new()
    } else {
        return Err(anyhow!("unsupported compiler {tool:?}"));
    };

    let output = tool
        .to_command()
        .current_dir(artifacts)
        .arg(code_path.strip_prefix(artifacts).unwrap())
        .args(args)
        .output()?;

    let exec_name = exec_name.to_string_lossy();

    if output.status.success() {
        let path = project_dir.join(artifacts).join(exec_name.as_ref());
        Ok(path)
    } else {
        let output = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            String::from_utf8_lossy(&output.stderr)
        }
        .to_string();

        Err(anyhow::anyhow!("compiler unsuccessful. output:\n{output}"))
    }
}
