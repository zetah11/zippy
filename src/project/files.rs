use std::collections::VecDeque;
use std::fs::{read_dir, ReadDir};
use std::io::Result;
use std::path::{Path, PathBuf};

/// Create an iterator which produces the path of every file in this directory
/// and any nested directories.
pub fn files(root: impl AsRef<Path>) -> Result<impl Iterator<Item = Result<PathBuf>>> {
    Ok(Files {
        queue: VecDeque::new(),
        current: read_dir(root)?,
    })
}

struct Files {
    queue: VecDeque<PathBuf>,
    current: ReadDir,
}

impl Iterator for Files {
    type Item = Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get the next entry in the current directory
            let entry = match self.current.next() {
                Some(entry) => entry,
                None => {
                    // Try again in the next directory if that fails
                    let next = self.queue.pop_front()?;
                    self.current = match read_dir(next) {
                        Ok(next) => next,
                        Err(e) => return Some(Err(e)),
                    };

                    continue;
                }
            };

            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => return Some(Err(e)),
            };

            let ft = match entry.file_type() {
                Ok(ft) => ft,
                Err(e) => return Some(Err(e)),
            };

            let path = entry.path();

            if ft.is_dir() {
                self.queue.push_back(path);
                continue;
            }

            return Some(Ok(path));
        }
    }
}
