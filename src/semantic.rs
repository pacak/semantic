//! Semantic document markup
//!
//! This module contains tools to generate documentation using semantic markup which can later be
//! rendered as [`markdown`](Semantic::render_to_markdown) or
//! [`manpage`](Semantic::render_to_manpage)
//!
//! Semantic document is composed of slices of (usually) styled text structured in possibly nested
//! blocks:
//! - section and subsection headers
//! - ordered, unordered and definitions lists, with items being nested blocks
//! - paragraphs of text
//! ```
//! # use roff::semantic::*;
//! let mut doc = Semantic::default();
//!
//! doc.section("Usage")
//!     .paragraph([text("Program takes "), literal("--help"), text(" flag")])
//!     .ulist(write_with(|doc| {
//!         doc.item(text("program is written in Rust"))
//!             .item(text("program should not crash"))
//!             .item([text("pass "), literal("--version"), text(" to see the version")]);
//!     }));
//! ```
//!
//! You can append any type that implements trait [`SemWrite`] using [`push`](Semantic::push).
//! `SemWrite` is implemented on any type that implements [`IntoIterator`] of other `SemWrite` items.
//!
//! You can also apply style to strings or characters using using [`literal`], [`metavar`],
//! [`mono`], [`text`] and [`important`].
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

pub use crate::{
    man::{Manpage, Section},
    shared::Style,
};
use crate::{monoid::FreeMonoid, roff::Font};
use std::{
    borrow::Cow,
    ops::{Add, AddAssign},
};

/// Semantic document that can be rendered
///
/// See [`module`](crate::semantic) documentation for more info
#[derive(Debug, Clone, Default)]
pub struct Semantic(FreeMonoid<Sem>);

impl AddAssign<&Self> for Semantic {
    fn add_assign(&mut self, rhs: &Self) {
        self.0 += &rhs.0;
    }
}

impl Add<&Self> for Semantic {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Semantic(self.0 + &rhs.0)
    }
}

impl Semantic {
    /// Insert document section name
    ///
    /// ```rust
    /// # use roff::semantic::*;
    /// let mut doc = Semantic::new();
    /// doc.section("Hello")
    ///     .paragraph("Some plain");
    ///
    /// ```
    pub fn section(&mut self, name: &str) -> &mut Self {
        self.push(Scoped(LogicalBlock::Section, text(name)))
    }

    /// Insert document subsection name
    pub fn subsection(&mut self, name: &str) -> &mut Self {
        self.push(Scoped(LogicalBlock::Subsection, text(name)))
    }

    /// Add a paragraph of text
    ///
    /// Paragraphs will be logically separated from each other by empty lines or indentation
    pub fn paragraph<S>(&mut self, text: S) -> &mut Self
    where
        S: SemWrite,
    {
        self.push(Scoped(LogicalBlock::Paragraph, text))
    }

    /// Insert a numbered list
    ///
    /// Items should contain one or more [`item`](Self::item) fragments
    pub fn nlist<S>(&mut self, items: S) -> &mut Self
    where
        S: SemWrite,
    {
        self.push(Scoped(LogicalBlock::NumberedList, items))
    }

    /// Insert an unnumbered list
    ///
    /// Items should contain one or more [`item`](Self::item) fragments
    pub fn ulist<S>(&mut self, items: S) -> &mut Self
    where
        S: SemWrite,
    {
        self.push(Scoped(LogicalBlock::UnnumberedList, items))
    }

    /// Insert a definition list
    ///
    /// Items should contain a combination of [`item`](Self::item), [`term`](Self::term) or
    /// [`definition`](Self::definition) fragments.
    pub fn dlist<S>(&mut self, items: S) -> &mut Self
    where
        S: SemWrite,
    {
        self.push(Scoped(LogicalBlock::DefinitionList, items))
    }

    /// Insert a list item
    ///
    /// Contents should be text level fragments, for [`definition
    /// lists`](Semantic::dlist) this will be used in the term body field.
    pub fn item<S>(&mut self, item: S) -> &mut Self
    where
        S: SemWrite,
    {
        self.push(Scoped(LogicalBlock::ListItem, item))
    }

    /// Insert a term into a definition list
    ///
    /// Contents should be a text level fragments
    pub fn term<T>(&mut self, term: T) -> &mut Self
    where
        T: SemWrite,
    {
        self.push(Scoped(LogicalBlock::ListKey, term))
    }

    /// Insert a definition into a definition list
    ///
    /// Combines both [`item`](Self::item) and [`term`](Self::term)
    pub fn definition<T, D>(&mut self, term: T, definition: D) -> &mut Self
    where
        T: SemWrite,
        D: SemWrite,
    {
        self.push(Scoped(LogicalBlock::ListKey, term));
        self.push(Scoped(LogicalBlock::ListItem, definition));
        self
    }

    /// Append
    #[inline(always)]
    pub fn push<S>(&mut self, text: S) -> &mut Self
    where
        S: SemWrite,
    {
        text.sem_write(self);
        self
    }

    pub fn mono<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(mono(payload.as_ref()))
    }

    pub fn literal<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(literal(payload.as_ref()))
    }

    pub fn metavar<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(metavar(payload.as_ref()))
    }

    pub fn text<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(text(payload.as_ref()))
    }

    pub fn important<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(important(payload.as_ref()))
    }
}

/// A semantic document fragment that can be appended to [`Semantic`] document
///
/// Semantic documents are designed with composing them from arbitrary typed chunks, not just
/// styled text. For example if document talks about command line option it should be possible to
/// insert this option by referring to a parser rather than by a string so documentation becomes
/// checked with a compiler
pub trait SemWrite {
    /// Append a fragment of semantic document
    fn sem_write(self, to: &mut Semantic);
}

struct SemWriteFn<F>(F);
impl<F> SemWrite for SemWriteFn<F>
where
    F: Fn(&mut Semantic),
{
    fn sem_write(self, to: &mut Semantic) {
        (self.0)(to);
    }
}

/// Helper method to combine several semantic fragments in a single write operation
///
/// If you are trying to create a paragraph of text from several fragments of different types you
/// can do something like this:
///
/// ```rust
/// # use roff::semantic::*;
/// let mut doc = Semantic::default();
/// doc.paragraph(write_with(|doc| {
///     doc.push([literal('-'), literal('h')]);
///     doc.push([text(" and "), literal("--help"), text(" prints usage")]);
/// }));
/// let doc = doc.render_to_markdown();
///
/// let expected = "<tt><b>\\-h</b></tt> and <tt><b>\\-\\-help</b></tt> prints usage";
/// assert_eq!(doc, expected);
/// ```
pub fn write_with<F>(action: F) -> impl SemWrite
where
    F: Fn(&mut Semantic) + Sized,
{
    SemWriteFn(action)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Sem {
    BlockStart(LogicalBlock),
    BlockEnd(LogicalBlock),
    Style(Style),
}

/// Logical block of text
///
/// List items are nested within lists, otherwise they should go on the top level
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum LogicalBlock {
    /// Section header
    Section,
    /// Subsection header
    Subsection,
    /// A paragraph of text - in general text should not go into the doc as is
    Paragraph,

    /// Unnumbered list, put `ListItem` inside
    UnnumberedList,
    /// Numbered list, put `ListItem` inside
    NumberedList,

    /// Definition list, should contain sequence of `ListKey` and `ListItem` pairs
    DefinitionList,

    /// List key, used inside `DefinitionList` only
    ListKey,

    /// List items, go in all types of lists
    ListItem,
}

/*
// -------------------------------------------------------------
impl<S> AddAssign<S> for Semantic
where
    S: SemWrite,
{
    fn add_assign(&mut self, rhs: S) {
        rhs.sem_write(self);
    }
}*/

/// Textual semantic fragment with attached style
///
/// Create it with [`literal`], [`metavar`] and similar methods. Contents inside of it are not
/// limited to string slices and document structure is optimized such that two sequential
/// styled writes with the same type produce the same results as a single write:
///
/// ```rust
/// # use roff::semantic::*;
/// // those two functions produce identical results, first one performs extra allocations
/// // while the second one is allocation free
/// fn write_long_name_1(doc: &mut Semantic, name: &str) {
///     doc.push(literal(format!("--{}", name)));
/// }
///
/// fn write_long_name_2(doc: &mut Semantic, name: &str) {
///     doc.push([literal("--"), literal(name)]);
/// }
/// ```
pub struct Styled<T>(Style, T);

impl SemWrite for Styled<&str> {
    fn sem_write(self, to: &mut Semantic) {
        to.0.squash = true;
        to.0.push_str(Sem::Style(self.0), self.1);
    }
}

impl SemWrite for Styled<Cow<'_, str>> {
    fn sem_write(self, to: &mut Semantic) {
        to.0.squash = true;
        to.0.push_str(Sem::Style(self.0), &self.1);
    }
}

impl SemWrite for Styled<String> {
    fn sem_write(self, to: &mut Semantic) {
        to.0.squash = true;
        to.0.push_str(Sem::Style(self.0), &self.1);
    }
}

impl SemWrite for Styled<char> {
    fn sem_write(self, to: &mut Semantic) {
        to.0.squash = true;
        to.0.push(Sem::Style(self.0), self.1);
    }
}

/// Literal semantic fragment
///
/// This fragment represents something user needs to type literally, usually used for command names
/// or option flag names:
///
/// ```rust
/// # use roff::semantic::*;
/// let mut doc = Semantic::default();
/// doc.push([text("Pass "), literal("--help"), text(" to print the usage")]);
/// let doc = doc.render_to_markdown();
/// let expected = "Pass <tt><b>\\-\\-help</b></tt> to print the usage";
///
/// assert_eq!(doc, expected);
/// ```
pub fn literal<T>(payload: T) -> Styled<T> {
    Styled(Style::Literal, payload)
}

/// Metavariable semantic fragment
///
/// This fragment represents something user needs to replace with a different input, usually used for
/// argument file name placeholders:
///
/// ```rust
/// # use roff::semantic::*;
/// let mut doc = Semantic::default();
/// doc.push([text("To save output to file: "), literal("-o"), mono(" "), metavar("FILE")]);
/// let doc = doc.render_to_markdown();
/// let expected = "To save output to file: <tt><b>\\-o</b></tt><tt> </tt><tt><i>FILE</i></tt>";
///
/// assert_eq!(doc, expected);
/// ```
pub fn metavar<T>(payload: T) -> Styled<T> {
    Styled(Style::Metavar, payload)
}

/// Plain text semantic fragment
///
/// This fragment should be used for any boring plaintext fragments:
///
/// ```rust
/// # use roff::semantic::*;
/// let mut doc = Semantic::default();
/// doc.push([text("To save output to file: "), literal("-o"), mono(" "), metavar("FILE")]);
/// let doc = doc.render_to_markdown();
/// let expected = "To save output to file: <tt><b>\\-o</b></tt><tt> </tt><tt><i>FILE</i></tt>";
///
/// assert_eq!(doc, expected);
/// ```
pub fn text<T>(payload: T) -> Styled<T> {
    Styled(Style::Text, payload)
}

/// Monospaced text semantic fragment
///
/// Can be useful to insert fixed text fragments for formatting or semantic emphasis
pub fn mono<T>(payload: T) -> Styled<T> {
    Styled(Style::Mono, payload)
}

/// Important text semantic fragment
///
/// Can be useful for any text that should attract users's attention
pub fn important<T>(payload: T) -> Styled<T> {
    Styled(Style::Important, payload)
}

struct Scoped<T>(pub LogicalBlock, pub T);
impl<S> SemWrite for Scoped<S>
where
    S: SemWrite,
{
    fn sem_write(self, to: &mut Semantic) {
        to.0.squash = false;
        to.0.push_str(Sem::BlockStart(self.0), "");
        self.1.sem_write(to);
        to.0.squash = false;
        to.0.push_str(Sem::BlockEnd(self.0), "");
    }
}

// -------------------------------------------------------------
impl<S, I> SemWrite for I
where
    S: SemWrite,
    I: IntoIterator<Item = S>,
{
    fn sem_write(self, to: &mut Semantic) {
        for s in self {
            s.sem_write(to);
        }
    }
}

/// Make it so new text is inserted at a new line
fn at_newline(res: &mut String) {
    if !(res.is_empty() || res.ends_with('\n')) {
        res.push('\n');
    }
}

/// Make it so new text is separated by an empty line
fn blank_line(res: &mut String) {
    if !(res.is_empty() || res.ends_with("\n\n")) {
        at_newline(res);
        res.push('\n');
    }
}

impl Semantic {
    /// Render semantic document into markdown
    // not quite markdown but encasing things in html block items makes it so
    // rustdoc avoids replacing -- to unicode dash - a nice side effect to have
    #[must_use]
    #[allow(clippy::too_many_lines)] // not that many
    pub fn render_to_markdown(&self) -> String {
        let mut res = String::new();

        // Items inside definition lists are encased in <dd> instead of <li>
        let mut is_dlist = false;
        for (meta, payload) in &self.0 {
            match meta {
                Sem::BlockStart(block) => match block {
                    LogicalBlock::DefinitionList => {
                        blank_line(&mut res);
                        is_dlist = true;
                        res.push_str("<dl>");
                    }
                    LogicalBlock::NumberedList => {
                        blank_line(&mut res);
                        is_dlist = false;
                        res.push_str("<ol>");
                    }
                    LogicalBlock::UnnumberedList => {
                        blank_line(&mut res);
                        is_dlist = false;
                        res.push_str("<ul>");
                    }
                    LogicalBlock::ListItem => {
                        at_newline(&mut res);
                        if is_dlist {
                            res.push_str("<dd>");
                        } else {
                            res.push_str("<li>");
                        }
                    }
                    LogicalBlock::ListKey => {
                        at_newline(&mut res);
                        res.push_str("<dt>");
                    }
                    LogicalBlock::Paragraph => {
                        blank_line(&mut res);
                        res.push_str("<p>");
                    }
                    LogicalBlock::Section => {
                        blank_line(&mut res);
                        res.push_str("# ");
                    }
                    LogicalBlock::Subsection => {
                        blank_line(&mut res);
                        res.push_str("## ");
                    }
                },
                Sem::BlockEnd(block) => match block {
                    LogicalBlock::DefinitionList => res.push_str("</dl>"),
                    LogicalBlock::UnnumberedList => res.push_str("</ul>"),
                    LogicalBlock::NumberedList => res.push_str("</ol>"),
                    LogicalBlock::ListItem => {
                        if is_dlist {
                            res.push_str("</dd>");
                        } else {
                            res.push_str("</li>");
                        }
                    }
                    LogicalBlock::ListKey => res.push_str("</dt>"),
                    LogicalBlock::Paragraph => res.push_str("</p>"),
                    LogicalBlock::Section | LogicalBlock::Subsection => {}
                },
                Sem::Style(style) => match style {
                    Style::Literal => {
                        res.push_str("<tt><b>");
                        res.push_str(payload);
                        res.push_str("</b></tt>");
                    }
                    Style::Metavar => {
                        res.push_str("<tt><i>");
                        res.push_str(payload);
                        res.push_str("</i></tt>");
                    }
                    Style::Mono => {
                        res.push_str("<tt>");
                        res.push_str(payload);
                        res.push_str("</tt>");
                    }
                    Style::Text => {
                        res.push_str(payload);
                    }
                    Style::Important => {
                        res.push_str("<b>");
                        res.push_str(payload);
                        res.push_str("</b>");
                    }
                },
            }
        }
        res
    }

    /// Render semantic document into a manpage
    ///
    /// You need to provide a [`Manpage`] to be used as a header
    #[must_use]
    pub fn render_to_manpage(&self, mut manpage: Manpage) -> String {
        // sections and subsections are implemented with .SH and .SS
        // control messages and it is easier to provide them right away
        // We also strip styling from them and change sections to all caps
        let mut capture = (String::new(), false);
        for (meta, payload) in &self.0 {
            match meta {
                Sem::BlockStart(b) => match b {
                    LogicalBlock::Section | LogicalBlock::Subsection => {
                        capture.1 = true;
                    }
                    LogicalBlock::UnnumberedList
                    | LogicalBlock::NumberedList
                    | LogicalBlock::Paragraph
                    | LogicalBlock::DefinitionList => {}
                    LogicalBlock::ListItem => {
                        manpage.raw().strip_newlines(true);
                    }
                    LogicalBlock::ListKey => {
                        manpage
                            .raw()
                            .control("TP", None::<&str>)
                            .strip_newlines(true);
                    }
                },
                Sem::BlockEnd(b) => match b {
                    LogicalBlock::Paragraph => {
                        manpage.raw().control("PP", None::<&str>);
                    }
                    LogicalBlock::Section => {
                        capture.1 = false;
                        manpage.section(&capture.0.to_uppercase());
                        capture.0.clear();
                    }
                    LogicalBlock::Subsection => {
                        capture.1 = false;
                        manpage.subsection(&capture.0);
                        capture.0.clear();
                    }
                    LogicalBlock::UnnumberedList => todo!(),
                    LogicalBlock::NumberedList => todo!(),
                    LogicalBlock::DefinitionList => {}
                    LogicalBlock::ListItem => {
                        manpage
                            .raw()
                            .control("PP", None::<&str>)
                            .strip_newlines(false);
                    }
                    LogicalBlock::ListKey => {
                        manpage.raw().roff_linebreak().strip_newlines(false);
                    }
                },
                Sem::Style(_) if capture.1 => {
                    capture.0.push_str(payload);
                }
                Sem::Style(s) => match s {
                    Style::Literal => {
                        manpage.raw().text([(Font::MonoBold, payload)]);
                    }
                    Style::Metavar => {
                        manpage.raw().text([(Font::Italic, payload)]);
                    }
                    Style::Mono => {
                        manpage.raw().text([(Font::Mono, payload)]);
                    }
                    Style::Text => {
                        manpage.raw().text([(Font::Roman, payload)]);
                    }
                    Style::Important => {
                        manpage.raw().text([(Font::Bold, payload)]);
                    }
                },
            }
        }

        manpage.render()
    }
}
