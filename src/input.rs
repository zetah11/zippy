use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_file(path: &Path) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    File::open(path)?.read_to_end(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}
