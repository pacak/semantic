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
//! doc.section("Section name");
//!
//! // hmmm not sure which one is better
//! doc += paragraph([text("Program takes "), literal("--help"), text(" argument")]);
//! doc.paragraph([text("Program takes "), literal("--help"), text(" argument")]);
//! ```
//!
//! You can append any type that implements trait [`SemWrite`] using `+=`, notable examples of such
//! types are [`Styled`], [`Scoped`] and [`WithScope`]. `SemWrite` is implemented on any type that
//! implements [`IntoIterator`] of other `SemWrite` items.
//!
//! You can also apply style to strings or characters using using [`literal`], [`metavar`],
//! [`mono`], [`text`] and [`important`].
//!

use crate::roff::Font;
pub use crate::{man::Manpage, monoid::FreeMonoid};
use std::ops::{Add, AddAssign};

/// Semantic document
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
    pub fn section(&mut self, name: &str) -> &mut Self {
        *self += Scoped(Block::Section, Styled(Style::Text, name));
        self
    }

    /// Insert document subsection name
    pub fn subsection(&mut self, name: &str) -> &mut Self {
        *self += Scoped(Block::Subsection, Styled(Style::Text, name));
        self
    }

    /// Add a paragraph of text
    ///
    /// Paragraphs will be logically separated from each other by empty lines or indentation
    pub fn paragraph<S>(&mut self, text: S) -> &mut Self
    where
        S: SemWrite,
    {
        *self += paragraph(text);
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Sem {
    BlockStart(Block),
    BlockEnd(Block),
    Style(Style),
}

pub use crate::shared::{Section, Style};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ListStyle {
    Numbered,
    Unnumbered,
    Definition,
}

impl From<ListStyle> for Block {
    fn from(value: ListStyle) -> Self {
        match value {
            ListStyle::Numbered => Block::NumberedList,
            ListStyle::Unnumbered => Block::UnnumberedList,
            ListStyle::Definition => Block::DefinitionList,
        }
    }
}

pub fn list<T, I>(style: ListStyle, items: I) -> impl SemWrite
where
    T: SemWrite,
    I: IntoIterator<Item = T>,
{
    Scoped(
        Block::from(style),
        items.into_iter().map(|i| Scoped(Block::ListItem, i)),
    )
}

impl<K, V> SemWrite for KeyVal<K, V>
where
    K: SemWrite,
    V: SemWrite,
{
    fn sem_write(self, to: &mut Semantic) {
        *to += Scoped(Block::ListKey, self.key);
        *to += Scoped(Block::ListKey, self.val);
    }
}
struct KeyVal<K, V> {
    key: K,
    val: V,
}

impl From<Style> for Sem {
    fn from(value: Style) -> Self {
        Sem::Style(value)
    }
}

/// Logical block of text
///
/// List items are nested within lists, otherwise they should go on the top level
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Block {
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

pub use semwrite::*;
mod semwrite {
    use super::*;

    pub trait SemWrite {
        fn sem_write(self, to: &mut Semantic);
    }

    // -------------------------------------------------------------
    impl<S> AddAssign<S> for Semantic
    where
        S: SemWrite,
    {
        fn add_assign(&mut self, rhs: S) {
            rhs.sem_write(self);
        }
    }

    // -------------------------------------------------------------
    pub struct Styled<T>(pub Style, pub T);

    // I want to be able to write styled strings, characters and iterators
    //
    // doc += norm("asdf");
    // doc += switch('-');
    // doc += switch('v');
    //
    // doc += [switch('-'), switch('v')];
    // doc += [switch("--"), switch("verbose")];
    //

    impl SemWrite for Styled<&str> {
        fn sem_write(self, to: &mut Semantic) {
            to.0.squash = true;
            to.0.push_str(Sem::Style(self.0), self.1);
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

    pub fn literal<T>(payload: T) -> Styled<T> {
        Styled(Style::Literal, payload)
    }

    pub fn metavar<T>(payload: T) -> Styled<T> {
        Styled(Style::Metavar, payload)
    }

    pub fn mono<T>(payload: T) -> Styled<T> {
        Styled(Style::Mono, payload)
    }

    pub fn text<T>(payload: T) -> Styled<T> {
        Styled(Style::Text, payload)
    }

    pub fn important<T>(payload: T) -> Styled<T> {
        Styled(Style::Important, payload)
    }

    pub fn paragraph<T>(payload: T) -> Scoped<T> {
        Scoped(Block::Paragraph, payload)
    }

    // -------------------------------------------------------------
    pub struct Scoped<T>(pub Block, pub T);
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
    pub struct WithScope<F>(pub Block, pub F);
    impl<F> SemWrite for WithScope<F>
    where
        F: Fn(&mut Semantic),
    {
        fn sem_write(self, to: &mut Semantic) {
            to.0.squash = false;
            to.0.push_str(Sem::BlockStart(self.0), "");
            self.1(to);
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
}

impl Semantic {
    pub fn render_to_markdown(&self) -> String {
        let mut res = String::new();
        let mut definition_list = false;
        let mut escape_dash = false;
        for (meta, payload) in &self.0 {
            match meta {
                Sem::BlockStart(block) => res.push_str(match block {
                    Block::DefinitionList => {
                        definition_list = true;
                        "<dl>"
                    }
                    Block::ListItem => {
                        if definition_list {
                            "<dd>"
                        } else {
                            "<li>"
                        }
                    }
                    Block::ListKey => "<dt>",
                    Block::NumberedList => {
                        definition_list = false;
                        "<ol>"
                    }
                    Block::Paragraph => {
                        escape_dash = true;
                        "\n\n"
                    }
                    Block::Section => "\n\n# ",
                    Block::Subsection => "\n\n## ",
                    Block::UnnumberedList => {
                        definition_list = false;
                        "<ul>"
                    }
                }),
                Sem::BlockEnd(block) => res.push_str(match block {
                    Block::DefinitionList => "</dl>\n",
                    Block::ListItem => {
                        if definition_list {
                            "</dd>\n"
                        } else {
                            "</li>\n"
                        }
                    }
                    Block::ListKey => "</dt>\n",
                    Block::NumberedList => "</ol>\n",
                    Block::Paragraph => {
                        escape_dash = false;
                        "\n\n"
                    }
                    Block::Section => "\n\n",
                    Block::Subsection => "\n\n",
                    Block::UnnumberedList => "</ul>\n",
                }),
                Sem::Style(style) => match style {
                    Style::Literal => {
                        res.push_str("<tt><b>");
                        for c in payload.chars() {
                            if c == '-' && escape_dash {
                                res.push('\\');
                            }
                            res.push(c);
                        }
                        res.push_str("</b></tt>");
                    }
                    Style::Metavar => {
                        res.push_str("<tt><i>");
                        res.push_str(payload);
                        res.push_str("</i></tt>");
                    }
                    Style::Mono => {
                        res.push_str("<tt>");
                        for c in payload.chars() {
                            if c == '-' && escape_dash {
                                res.push('\\');
                            }
                            res.push(c);
                        }
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

    pub fn render_to_manpage(&self, title: &str, section: Section, extra: &[&str]) -> String {
        let mut res = Manpage::new(title, section, extra);

        let mut capture = (String::new(), false);
        for (meta, payload) in &self.0 {
            match meta {
                Sem::BlockStart(b) => match b {
                    Block::Paragraph => {
                        res.raw().strip_newlines(true);
                    }
                    Block::Section => {
                        capture.1 = true;
                    }
                    Block::Subsection => {
                        capture.1 = true;
                    }
                    Block::UnnumberedList => todo!(),
                    Block::NumberedList => todo!(),
                    Block::DefinitionList => {}
                    Block::ListItem => {}
                    Block::ListKey => {
                        res.raw().control("TP", None::<&str>).strip_newlines(true);
                    }
                },
                Sem::BlockEnd(b) => match b {
                    Block::Paragraph => {
                        res.raw().strip_newlines(false).control("PP", None::<&str>);
                    }
                    Block::Section => {
                        capture.1 = false;
                        res.section(&capture.0);
                        capture.0.clear();
                    }
                    Block::Subsection => {
                        capture.1 = false;
                        res.subsection(&capture.0);
                        capture.0.clear();
                    }
                    Block::UnnumberedList => todo!(),
                    Block::NumberedList => todo!(),
                    Block::DefinitionList => {}
                    Block::ListItem => {
                        res.raw().control("PP", None::<&str>).strip_newlines(false);
                    }
                    Block::ListKey => {
                        res.raw().roff_linebreak();
                    }
                },
                Sem::Style(_) if capture.1 => {
                    capture.0.push_str(payload);
                }
                Sem::Style(s) => match s {
                    Style::Literal => {
                        res.raw().text([(Font::MonoBold, payload)]);
                    }
                    Style::Metavar => {
                        res.raw().text([(Font::Italic, payload)]);
                    }
                    Style::Mono => {
                        res.raw().text([(Font::Mono, payload)]);
                    }
                    Style::Text => {
                        res.raw().text([(Font::Roman, payload)]);
                    }
                    Style::Important => {
                        res.raw().text([(Font::Bold, payload)]);
                    }
                },
            }
        }

        res.render()
    }
}

// document consist of multiple sections
// each section can contain section level elements:
// - subsections
// - paragraphs
// - lists
// - definition lists
//
//
// each subsection can contain same things as section minus other subsections
//
// - paragraphs, lists and definition lists  contain inline elements:
//
//
// Inline elements are
// - plain text
// - various flag things:
//   optional/required/many/or flags, metavar, command
//
