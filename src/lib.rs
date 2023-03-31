#![warn(missing_docs)]

//! Semantic document markup
//!
//! This crate contains tools to generate documentation using semantic markup which can later be
//! rendered as [`markdown`](Doc::render_to_markdown) or
//! [`manpage`](Doc::render_to_manpage)
//!
//! Semantic document is composed of slices of (usually) styled text structured in possibly nested
//! blocks:
//! - section and subsection headers
//! - ordered, unordered and definitions lists, with items being nested blocks
//! - paragraphs of text
//! ```
//! # use ::roff::*;
//! let mut doc = Doc::default();
//!
//! doc.section("Usage")
//!     .paragraph([text("Program takes "), literal("--help"), text(" flag")])
//!     .ulist(|doc: &mut Doc| {
//!         doc.item(text("program is written in Rust"))
//!             .item(text("program should not crash"))
//!             .item([text("pass "), literal("--version"), text(" to see the version")]);
//!     });
//! ```
//!
//! <details>
//! <summary>Generated structure</summary>
//! <style>
//!     .sem_border * {
//!         padding: 5px 10px;
//!         margin: 5px;
//!         overflow: auto;
//!         border: 2px solid;
//!     }
//!     .sem_border span {
//!         display: inline-block;
//!         margin: 0px;
//!         padding: 2px 10px;
//!         background: var(--code-block-background-color);
//!         border: 2px dotted;
//!         font-color: black;
//!     }
//! </style>
//!
//! <div class="sem_border" style="padding: 5px; border: 1px dashed grey;">
//!   <div style="border-color: blue;">
//!     Usage
//!   </div>
//!
//!   <div style="border-color: red;">
//!     <span style="border-color: blue;">Program takes </span>
//!     <span style="border-color: green;">--help</span>
//!     <span style="border-color: blue;"> flag</span>
//!   </div>
//!
//!   <div style="border-color: cyan;">
//!     <div style="border-color: purple;">
//!       <span style="border-color: blue;">program is written in rust</span>
//!     </div>
//!     <div style="border-color: purple;">
//!       <span style="border-color: blue;">program should not crash</span>
//!     </div>
//!     <div style="border-color: purple;">
//!       <span style="border-color: blue;">pass </span>
//!       <span style="border-color: green;">--version</span>
//!       <span style="border-color: blue;"> to see the version</span>
//!     </div>
//!   </div>
//!
//!
//! </div>
//! </details>
//!
//! <details>
//! <summary>Rendered markdown</summary>
//!
//! # Usage
//!
//! <p>Program takes <tt><b>--help</b></tt> flag</p>
//!
//! <ul>
//! <li>program is written in Rust</li>
//! <li>program should not crash</li>
//! <li>pass <tt><b>--version</b></tt> to see the version</li>
//! </ul>
//! </details>

mod escape;
mod monoid;
#[doc(hidden)]
pub mod roff;
mod semantic;
mod shared;

#[doc(inline)]
pub use crate::{semantic::*, shared::*};

use std::path::Path;

/// Update file contents if needed and return if it was needed
///
/// # Example
///
/// One way to use this function would be to make a test like this so CI would fail if files
/// in the repository are outdated.
/// ```no_run
/// # /*
/// #[test]
/// fn update_documentation() {
/// # */
/// # use ::roff::write_updated;
///     // create some document
///     let doc = "hello world";
///
///     // write file, fail the test so CI fails if new documentation
///     // not checked in so repository is up to date if CI passes
///     assert!(
///         write_updated("path/to/file", doc.as_bytes()).unwrap(),
///         "Doc changes detected, you need to commit the output."
///     );
/// # /*
/// }
/// # */
/// ```
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
