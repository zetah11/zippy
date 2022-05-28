//! Provides conveniences for working with files, including maps for
//! assosciating filenames and source ids, as well as deducing language type
//! from file names.

mod map;

pub use map::FilenameMap;

use std::path::Path;

use zc::lang::Language;

/// Get whether the given uri represents a static or dynamic file. Static is the
/// default if the extension is not one of `z` (static), `zs` (static) or `zd`
/// (dynamic).
pub fn uri_to_language(uri: &String) -> Language {
    let p = Path::new(uri);
    match p.extension() {
        Some(ext) => match ext.to_str() {
            Some("zd") => Language::Dynamic,
            _ => Language::Static,
        },
        _ => Language::Static,
    }
}
