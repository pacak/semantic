#![warn(missing_docs)]
//!
//! This library implements several interfaces for generating documentation:
//!
//! - [`roff`] - low level API designed for generating arbitrary ROFF
//! - [`man`] - higher level API designed for generating man pages
//! - [`semantic`] - highest level API designed for generating documentation in general in both
//! roff and markdown formats
//!

use std::path::Path;

mod escape;
pub mod man;
mod monoid;
pub mod roff;
pub mod semantic;
mod shared;

/// Update file contents if needed and return if it was needed
///
/// # Errors
/// Reports any file IO errors
pub fn write_updated<P: AsRef<Path>>(path: P, value: &[u8]) -> std::io::Result<bool> {
    use std::fs::OpenOptions;
    use std::io::{Read, Seek};
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)?;
    let mut current_val = Vec::new();
    file.read_to_end(&mut current_val)?;
    if current_val == value {
        Ok(false)
    } else {
        file.set_len(0)?;
        file.seek(std::io::SeekFrom::Start(0))?;
        std::io::Write::write_all(&mut file, value)?;
        Ok(true)
    }
}
