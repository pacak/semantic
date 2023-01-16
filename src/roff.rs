//! Low level interface for Roff composer
//!
//! [ROFF] is a family of Unix text-formatting languages, implemented
//! by the `nroff`, `troff`, and `groff` programs, among others. See
//! [groff(7)] for a description of the language. This structure is an
//! abstract representation of a document in ROFF format. It is meant
//! for writing code to generate ROFF documents, such as manual pages.
//!
//! # Example
//! ```rust
//! # use ::roff::roff::*;
//! let doc = Roff::new()
//!     .text([(Font::Roman, "hello, world")])
//!     .render(Apostrophes::DontHandle);
//! assert_eq!(doc, "\\fRhello, world\\fP");
//! ```
//!
//! [ROFF]: https://en.wikipedia.org/wiki/Roff_(software)

use std::ops::{Add, AddAssign};

pub use crate::escape::Apostrophes;
use crate::{escape::Escape, monoid::FreeMonoid};

/// A Roff document with a low level interface
///
/// # Example
/// ```rust
/// # use ::roff::roff::*;
/// let doc = Roff::new()
///     .control("TH", ["FOO", "1"])
///     .control("SH", ["NAME"])
///     .text([(Font::Current, "foo - do a foo thing")])
///     .render(Apostrophes::DontHandle);
/// assert_eq!(doc, ".TH FOO 1\n.SH NAME\nfoo \\- do a foo thing");
/// ```
#[derive(Debug, Default, Clone)]
pub struct Roff {
    payload: FreeMonoid<Escape>,
    /// keep or strip newlines from inserted text
    pub strip_newlines: bool,
}

/// Font selector
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Font {
    /// Currently selected font,
    Current,

    /// Roman font,
    Roman,

    /// Bold font
    Bold,

    /// Italic font
    Italic,

    /// A font that is both bold and italic
    BoldItalic,

    /// Regular constant width font, same as `Regular` in terminal output
    Mono,

    /// Bold constant width font, same as just `Bold` in terminal output
    MonoBold,

    /// Italic constant width font, same as just `Italic` in terminal output
    MonoItalic,
}

/// Escape code used to return to the previous font
pub(crate) const RESTORE_FONT: &str = "\\fP";

impl Font {
    /// Escape sequence needed to set this font, None for default font
    ///
    pub(crate) fn escape(self) -> Option<&'static str> {
        match self {
            Font::Bold => Some("\\fB"),
            Font::BoldItalic => Some("\\f(BI"),
            Font::Current => None,
            Font::Italic => Some("\\fI"),
            Font::Mono => Some("\\f(CR"),
            Font::MonoBold => Some("\\f(CB"),
            Font::MonoItalic => Some("\\f(CI"),
            Font::Roman => Some("\\fR"),
        }
    }
}

impl Roff {
    /// Create new raw Roff document
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Chainable setter for `strip_newlines` field
    ///
    /// `strip_newlines` specifies if [`render`](Self::render) should keep all the newline characters
    /// inside added text newlines can come with a special meaning, for example adding a section
    /// header relies, newlines are automatically stripped from [`control`](Self::control) arguments.
    ///
    /// ```rust
    /// # use roff::roff::*;
    /// let doc = Roff::new()
    ///     .plaintext("this newline is kept\n")
    ///     .strip_newlines(true)
    ///     .plaintext("but this one\nis removed.")
    ///     .render(Apostrophes::DontHandle);
    /// assert_eq!(doc, "this newline is kept\nbut this one is removed.");
    /// ```
    pub fn strip_newlines(&mut self, state: bool) -> &mut Self {
        self.strip_newlines = state;
        self
    }

    /// Remove all the contents
    pub fn clear(&mut self) {
        self.payload.clear();
    }

    /// Size of textual part of the payload, in bytes.
    ///
    /// Rendered output will include all the control sequences so most likely will be bigger
    #[must_use]
    pub fn len(&self) -> usize {
        self.payload.len()
    }

    /// Check if document is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }

    /// Insert a raw control sequence
    ///
    /// `name` should not contain initial `'.'`.
    /// Arguments are taken from an iterator and escaped accordingly
    ///
    /// ```rust
    /// # use ::roff::roff::*;
    /// let doc = Roff::new()
    ///     .control("SH", ["Section\nname with newline"])
    ///     .render(Apostrophes::DontHandle);
    /// assert_eq!(doc, ".SH Section\\ name\\ with\\ newline\n");
    /// ```
    ///
    /// For control sequences that take no arguments you can pass `None::<&str>`
    /// ```rust
    /// # use ::roff::roff::*;
    /// let doc = Roff::new()
    ///     .control("PP", None::<&str>)
    ///     .render(Apostrophes::DontHandle);
    /// assert_eq!(doc, ".PP\n");
    /// ```
    pub fn control<S, I>(&mut self, name: &str, args: I) -> &mut Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        self.payload.push_str(Escape::UnescapedAtNewline, ".");
        self.payload.push_str(Escape::Unescaped, name);
        for arg in args {
            self.payload
                .push_str(Escape::Unescaped, " ")
                .push_str(Escape::Spaces, arg.as_ref());
        }
        self.payload.push_str(Escape::UnescapedAtNewline, "");
        self
    }

    /// Insert a line break in the Roff document source
    ///
    /// This will not show up in the output of the roff program.
    pub fn roff_linebreak(&mut self) -> &mut Self {
        self.payload.push_str(Escape::UnescapedAtNewline, "");
        self
    }

    /// Insert a comment in the Roff document source
    ///
    /// This will not show up in the output of the roff program.
    pub fn roff_comment(&mut self, text: &str) -> &mut Self {
        self.payload
            .push_str(Escape::UnescapedAtNewline, ".\\\" ")
            .push_str(Escape::SpecialNoNewline, text);
        self
    }

    /// Insert raw escape sequence
    ///
    /// You can use all the notations for the escapes, they will be copied into the output stream
    /// as is without extra checks or escapes.
    /// ```rust
    /// # use roff::roff::*;
    /// let text = Roff::new()
    ///     .escape("\\fB")
    ///     .plaintext("bold ")
    ///     .escape("\\f(CR")
    ///     .plaintext("mono bold ")
    ///     .escape("\\f[R]")
    ///     .plaintext("regular plaintext")
    ///     .render(Apostrophes::DontHandle);
    /// assert_eq!(text, "\\fBbold \\f(CRmono bold \\f[R]regular plaintext");
    /// ```
    pub fn escape(&mut self, arg: &str) -> &mut Self {
        self.payload.push_str(Escape::Unescaped, arg);
        self
    }

    /// Insert a plain text string, special characters are escaped
    ///
    /// ```rust
    /// # use roff::roff::*;
    /// let doc = Roff::new()
    ///     .plaintext(".some random text that starts with a dot")
    ///     .render(Apostrophes::DontHandle);
    /// assert_eq!(doc, "\\&.some random text that starts with a dot");
    /// ```
    pub fn plaintext(&mut self, text: &str) -> &mut Self {
        if self.strip_newlines {
            self.payload.push_str(Escape::SpecialNoNewline, text);
        } else {
            self.payload.push_str(Escape::Special, text);
        }
        self
    }

    /// Insert one or more string slices using custom font for each one
    ///
    /// ```rust
    /// # use roff::roff::*;
    /// let doc = Roff::default()
    ///     .text([
    ///         (Font::Roman, "You can add an "),
    ///         (Font::Bold, "emphasis"),
    ///         (Font::Roman, " to some words."),
    ///     ])
    ///     .render(Apostrophes::DontHandle);
    /// assert_eq!(doc, "\\fRYou can add an \\fBemphasis\\fR to some words.\\fP");
    /// ```
    pub fn text<I, S>(&mut self, text: I) -> &mut Self
    where
        I: IntoIterator<Item = (Font, S)>,
        S: AsRef<str>,
    {
        let mut prev_font = None;
        for (font, item) in text {
            if prev_font == Some(font) {
                self.plaintext(item.as_ref());
            } else if let Some(escape) = font.escape() {
                self.escape(escape).plaintext(item.as_ref());
                prev_font = Some(font);
            } else {
                self.plaintext(item.as_ref());
            }
        }
        if prev_font.is_some() {
            self.escape(RESTORE_FONT);
        }
        self
    }

    /// Render Roff document to `String`
    ///
    /// This method creates a valid ROFF document which can be fed to a ROFF implementation
    #[must_use]
    pub fn render(&self, ap: Apostrophes) -> String {
        let mut res = Vec::with_capacity(self.payload.len() * 2);
        if ap == Apostrophes::Handle {
            res.extend(crate::escape::APOSTROPHE_PREABMLE.as_bytes());
        }
        crate::escape::escape(&self.payload, &mut res, ap);
        String::from_utf8(res).expect("Should be valid utf8 by construction")
    }
}

impl AddAssign<&Roff> for Roff {
    fn add_assign(&mut self, rhs: &Roff) {
        self.payload += &rhs.payload;
        self.strip_newlines = rhs.strip_newlines;
    }
}

impl Add<&Roff> for Roff {
    type Output = Self;

    fn add(mut self, rhs: &Roff) -> Self::Output {
        self += rhs;
        self
    }
}

#[cfg(test)]
mod test {
    use super::{Apostrophes, Font, Roff};
    const NO_AP: Apostrophes = Apostrophes::DontHandle;

    #[test]
    fn escape_dash_in_plaintext() {
        let text = Roff::default().plaintext("-").render(NO_AP);
        assert_eq!(r"\-", text);
    }

    #[test]
    fn escape_backslash_in_plaintext() {
        let text = Roff::default().plaintext(r"\x").render(NO_AP);
        assert_eq!(r"\\x", text);
    }

    #[test]
    fn escape_backslash_and_dash_in_plaintext() {
        let text = Roff::default().plaintext(r"\-").render(NO_AP);
        assert_eq!(r"\\\-", text);
    }

    #[test]
    fn escapes_leading_control_chars_and_space_in_plaintext() {
        let text = Roff::default()
            .plaintext("foo\n.bar\n'yo\n hmm")
            .render(NO_AP);
        assert_eq!("foo\n\\&.bar\n\\&'yo\n\\& hmm", text);
    }

    #[test]
    fn escape_plain_in_plaintext() {
        let text = Roff::default().plaintext("abc").render(NO_AP);
        assert_eq!("abc", text);
    }

    #[test]
    fn render_dash_in_plaintext() {
        let text = Roff::default().plaintext("foo-bar").render(NO_AP);
        assert_eq!("foo\\-bar", text);
    }

    #[test]
    fn render_dash_in_font() {
        let text = Roff::default()
            .text([(Font::Current, "foo-bar")])
            .render(NO_AP);
        assert_eq!("foo\\-bar", text);
    }

    #[test]
    fn render_roman() {
        let text = Roff::default().text([(Font::Current, "foo")]).render(NO_AP);
        assert_eq!("foo", text);
    }

    #[test]
    fn render_italic() {
        let text = Roff::default().text([(Font::Italic, "foo")]).render(NO_AP);
        assert_eq!("\\fIfoo\\fP", text);
    }

    #[test]
    fn render_bold() {
        let text = Roff::default().text([(Font::Bold, "foo")]).render(NO_AP);
        assert_eq!("\\fBfoo\\fP", text);
    }

    #[test]
    fn render_text_roman() {
        let text = Roff::default().text([(Font::Roman, "roman")]).render(NO_AP);
        assert_eq!("\\fRroman\\fP", text);
    }
    #[test]
    fn render_text_plain() {
        let text = Roff::default()
            .text([(Font::Current, "roman")])
            .render(NO_AP);
        assert_eq!("roman", text);
    }

    #[test]
    fn render_text_with_leading_period() {
        let text = Roff::default()
            .text([(Font::Current, ".roman")])
            .render(NO_AP);
        assert_eq!("\\&.roman", text);
    }

    #[test]
    fn render_text_with_newline_period() {
        let text = Roff::default()
            .text([(Font::Current, "foo\n.roman")])
            .render(NO_AP);
        assert_eq!("foo\n\\&.roman", text);
    }

    #[test]
    fn render_line_break() {
        let text = Roff::default()
            .text([(Font::Current, "roman\n")])
            .control("br", None::<&str>)
            .text([(Font::Current, "more\n")])
            .render(NO_AP);
        assert_eq!("roman\n.br\nmore\n", text);
    }

    #[test]
    fn render_control() {
        let text = Roff::default()
            .control("foo", ["bar", "foo and bar"])
            .render(NO_AP);
        assert_eq!(".foo bar foo\\ and\\ bar\n", text);
    }

    #[test]
    fn twice_bold() {
        let text = Roff::default()
            .text([
                (Font::Bold, "bold,"),
                (Font::Current, " more bold"),
                (Font::Bold, " and more bold"),
            ])
            .render(NO_AP);
        assert_eq!("\\fBbold, more bold and more bold\\fP", text);
    }

    #[test]
    fn multiple_controls() {
        let text = Roff::default()
            .control("br", None::<&str>)
            .control("br", None::<&str>)
            .control("br", None::<&str>)
            .render(NO_AP);
        assert_eq!(".br\n.br\n.br\n", text);
    }
}
