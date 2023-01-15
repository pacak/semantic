//! High level interface designed to generate manpages

//! # Example: generating a man page
//! ```rust
//! # use ::roff::{man::*, write_updated};
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
//! # use std::{path::PathBuf, env::var_os};
//! let path = PathBuf::from(var_os("CARGO_MANIFEST_DIR").unwrap()).join("corrupt.1");
//! write_updated(path, &page).unwrap();
//! # drop(page);
//! ```
//!
//! Which outputs
//! ```roff
//! .ie \n(.g .ds Aq \(aq
//! .el .ds Aq '
//! .TH \&CORRUPT 1
//! .SH \&NAME
//! \fRcorrupt \- modify files by randomly changing bits\fP
//! .PP
//! .SH \&SYNOPSIS
//! \fBcorrupt\fR [\fB\-n\fR \fIBITS\fR]\fP
//! .PP
//! .SH \&DESCRIPTION
//! \fBcorrupt\fR modifies files by toggling a randomly chosen bit.\fP
//! .PP
//! .SH \&OPTIONS
//! .TP
//! \fB\-n\fR=\fIBITS\fP
//! .PP
//! \fRSet the number of bits to modify\fP
//! .PP
//! ```
//!
//! Which will be shown by the `man(1)` command as:
//!
//! ```txt
//! CORRUPT(1)                    General Commands Manual                   CORRUPT(1)
//!
//! NAME
//!        corrupt - modify files by randomly changing bits
//!
//! SYNOPSIS
//!        corrupt [-n BITS]
//!
//! DESCRIPTION
//!        corrupt modifies files by toggling a randomly chosen bit.
//!
//! OPTIONS
//!        -n=BITS
//!
//!        Set the number of bits to modify
//!                                                                         CORRUPT(1)
//! ```
//!
//! [ROFF]: https://en.wikipedia.org/wiki/Roff_(software)
//! [groff(7)]: https://manpages.debian.org/bullseye/groff/groff.7.en.html
//! [man]: https://en.wikipedia.org/wiki/Man_page

use crate::roff::{Apostrophes, Font, Roff};

/// Manpage Roff document
#[derive(Debug, Clone)]
pub struct Manpage {
    roff: Roff,
}

/// Font style, unlike [`Font`](crate::roff::Font), this style focuses more on what to render rather
/// than how to render it
pub enum Style {
    /// Metavariables and other placeholder text
    Metavar,

    /// Switches and argument names, command names and other things user is expected to type
    /// implicitly
    Argument,

    /// Plain text with no decorations
    Normal,

    /// Extra highlighed text. You can also use `Metavar` or `Argument`.
    Highlight,
}

impl Style {
    pub(crate) fn font(&self) -> Font {
        match self {
            Style::Metavar => Font::Italic,
            Style::Argument => Font::Bold,
            Style::Normal => Font::Roman,
            Style::Highlight => Font::BoldItalic,
        }
    }
}

impl Manpage {
    /// Create a new manpage with given `title` in a given `section`
    ///
    /// `extra` can contain up to 3 items that will populate corners in header and footer lines
    /// - free form date when the application was last updated
    /// - if a program is a part of some project or a suite - it goes here
    /// - fancier, human readlable application name
    pub fn new<T>(title: T, section: Section, extra: &[&str]) -> Self
    where
        T: AsRef<str>,
    {
        let mut roff = Roff::default();
        roff.control(
            "TH",
            [title.as_ref(), section.as_str()]
                .iter()
                .chain(extra.iter().take(3)),
        );
        Self { roff }
    }

    /// Add a new unnumbered section
    pub fn section<S: AsRef<str>>(&mut self, title: S) -> &mut Self {
        self.roff.control("SH", &[title]);
        self
    }

    /// Add a new unnumbered subsection
    pub fn subsection<S: AsRef<str>>(&mut self, title: S) -> &mut Self {
        self.roff.control("SS", &[title]);
        self
    }

    /// Add a new indented label
    pub fn label<T, S>(&mut self, offset: Option<&str>, text: T) -> &mut Self
    where
        T: IntoIterator<Item = (Style, S)>,
        S: AsRef<str>,
    {
        let strip = self.roff.strip_newlines;
        self.roff
            .control("TP", offset)
            .strip_newlines(true)
            .text(text.into_iter().map(|pair| (pair.0.font(), pair.1)))
            .control("PP", None::<&str>)
            .strip_newlines(strip);
        self
    }

    /// Add a new paragraph
    pub fn paragraph<T, S>(&mut self, text: T) -> &mut Self
    where
        T: IntoIterator<Item = (Style, S)>,
        S: AsRef<str>,
    {
        self.roff
            .strip_newlines(true)
            .text(text.into_iter().map(|pair| (pair.0.font(), pair.1)))
            .control("PP", None::<&str>);
        self
    }

    /// Get a raw Roff stream
    pub fn raw(&mut self) -> &mut Roff {
        &mut self.roff
    }

    /// Render manpage
    #[must_use]
    pub fn render(&self) -> String {
        self.roff.render(Apostrophes::Handle)
    }
}

#[derive(Debug, Clone, Copy)]
/// Manpage section
pub enum Section<'a> {
    /// General commands
    General,
    /// System calls
    SystemCall,
    /// Library functions such as C standard library functions
    LibraryFunction,
    /// Special files (usually devices in /dev) and drivers
    SpecialFile,
    /// File formats and conventions
    FileFormat,
    /// Games and screensavers
    Game,
    /// Miscellaneous
    Misc,
    /// System administration commands and daemons
    Sysadmin,
    /// Custom section, must start with a digit 1 to 8, can have a string appended to indicate a
    /// subsection
    Custom(&'a str),
}

impl Section<'_> {
    fn as_str(&self) -> &str {
        match self {
            Section::General => "1",
            Section::SystemCall => "2",
            Section::LibraryFunction => "3",
            Section::SpecialFile => "4",
            Section::FileFormat => "5",
            Section::Game => "6",
            Section::Misc => "7",
            Section::Sysadmin => "8",
            Section::Custom(s) => s,
        }
    }
}
