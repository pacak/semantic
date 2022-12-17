#![warn(missing_docs)]
//! A document in the ROFF format.
//!
//! [ROFF] is a family of Unix text-formatting languages, implemented
//! by the `nroff`, `troff`, and `groff` programs, among others. See
//! [groff(7)] for a description of the language. This structure is an
//! abstract representation of a document in ROFF format. It is meant
//! for writing code to generate ROFF documents, such as manual pages.
//!
//! This library implements several interfaces for generating ROFF
//! documents:
//!
//! - [`raw`] - low level API designed for generating arbitrary ROFF
//! - [`man`] - high level API designed for generating man pages
//!
//! # Example: low level API
//!
//! ```rust
//! # use ::roff::raw::*;
//! let doc = Roff::new()
//!     .text([(Font::Roman, "hello, world")])
//!     .render(Apostrophes::DontHandle);
//! assert_eq!(doc, "\\fRhello, world\\fP");
//! ```
//!
//! # Example: generating a man page
//! ```rust
//! # use ::roff::man::*;
//! use Style::*;
//! let page = Manpage::new("CORRUPT", Section::General, &[])
//!     .section("NAME")
//!     .paragraph([(Normal, "corrupt - modify files by randomly changing bits")])
//!     .section("SYNOPSIS")
//!     .paragraph([
//!         (Argument, "corrupt"),
//!         (Normal, " ["), (Argument, "-n"), (Normal, " "), (Metavar, "BITS"), (Normal, "]"),
//!     ])
//!     .section("DESCRIPTION")
//!     .paragraph([
//!         (Argument, "corrupt"),
//!         (Normal, " modifies files by toggling a randomly chosen bit."),
//!     ])
//!     .section("OPTIONS")
//!     .label(None, [(Argument, "-n"), (Normal, "="), (Metavar, "BITS") ])
//!     .paragraph([(Normal, "Set the number of bits to modify")])
//!     .render();
//!
//! // write_updated(page, "corrupt.1").unwrap();
//! # drop(page);
//! ```
//!
//! [ROFF]: https://en.wikipedia.org/wiki/Roff_(software)
//! [groff(7)]: https://manpages.debian.org/bullseye/groff/groff.7.en.html
//! [man]: https://en.wikipedia.org/wiki/Man_page

use std::path::Path;

mod escape;
pub mod man;
pub mod mdoc;
mod monoid;
pub mod raw;

/// Update file contents if needed and return if it was needed
///
/// # Errors
/// Reports any file IO errors
pub fn write_updated<P: AsRef<Path>>(value: &str, path: P) -> std::io::Result<bool> {
    use std::fs::OpenOptions;
    use std::io::{Read, Seek};
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)?;
    let mut current_val = String::new();
    file.read_to_string(&mut current_val)?;
    if current_val == value {
        Ok(false)
    } else {
        file.set_len(0)?;
        file.seek(std::io::SeekFrom::Start(0))?;
        std::io::Write::write_all(&mut file, value.as_bytes())?;
        Ok(true)
    }
}
