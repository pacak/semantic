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
//! doc.section("Section name")
//!     .paragraph([text("Program takes "), literal("--help"), text(" argument")]);
//! ```
//!
//! You can append any type that implements trait [`SemWrite`] using `+=`, notable examples of such
//! types are [`Styled`], [`Scoped`] and [`WithScope`]. `SemWrite` is implemented on any type that
//! implements [`IntoIterator`] of other `SemWrite` items.
//!
//! You can also apply style to strings or characters using using [`literal`], [`metavar`],
//! [`mono`], [`text`] and [`important`].
//!

pub use crate::{
    man::Manpage,
    shared::{Section, Style},
};
use crate::{monoid::FreeMonoid, roff::Font};
use std::ops::{Add, AddAssign};

/// Semantic document that can be rendered
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
        *self += Scoped(Block::Paragraph, text);
        self
    }

    pub fn numbered_list<S>(&mut self, items: S) -> &mut Self
    where
        S: SemWrite,
    {
        *self += Scoped(Block::NumberedList, items);
        self
    }

    pub fn unnumbered_list<S>(&mut self, items: S) -> &mut Self
    where
        S: SemWrite,
    {
        *self += Scoped(Block::UnnumberedList, items);
        self
    }

    pub fn definition_list<S>(&mut self, items: S) -> &mut Self
    where
        S: SemWrite,
    {
        *self += Scoped(Block::DefinitionList, items);
        self
    }

    pub fn item<S>(&mut self, item: S) -> &mut Self
    where
        S: SemWrite,
    {
        *self += Scoped(Block::ListItem, item);
        self
    }

    pub fn term<T>(&mut self, term: T) -> &mut Self
    where
        T: SemWrite,
    {
        *self += Scoped(Block::ListKey, term);
        self
    }
    pub fn definition<T, D>(&mut self, term: T, definition: D) -> &mut Self
    where
        T: SemWrite,
        D: SemWrite,
    {
        *self += Scoped(Block::ListKey, term);
        *self += Scoped(Block::ListItem, definition);
        self
    }

    pub fn text<S>(&mut self, text: S) -> &mut Self
    where
        S: SemWrite,
    {
        *self += text;
        self
    }
}

struct SemWriteFn<F>(F);
impl<F> SemWrite for SemWriteFn<F>
where
    F: Fn(&mut Semantic),
{
    fn sem_write(self, to: &mut Semantic) {
        (self.0)(to)
    }
}

pub fn write_with<F>(action: F) -> impl SemWrite
where
    F: Fn(&mut Semantic) + Sized,
{
    SemWriteFn(action)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Sem {
    BlockStart(Block),
    BlockEnd(Block),
    Style(Style),
}

/// Logical block of text
///
/// List items are nested within lists, otherwise they should go on the top level
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Block {
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

/*
pub fn paragraph<T>(payload: T) -> Scoped<T> {
    Scoped(Block::Paragraph, payload)
}*/

// -------------------------------------------------------------
struct Scoped<T>(pub Block, pub T);
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

    pub fn render_to_manpage(&self, mut manpage: Manpage) -> String {
        let mut capture = (String::new(), false);
        for (meta, payload) in &self.0 {
            match meta {
                Sem::BlockStart(b) => match b {
                    Block::Paragraph => {
                        manpage.raw().strip_newlines(true);
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
                        manpage
                            .raw()
                            .control("TP", None::<&str>)
                            .strip_newlines(true);
                    }
                },
                Sem::BlockEnd(b) => match b {
                    Block::Paragraph => {
                        manpage
                            .raw()
                            .strip_newlines(false)
                            .control("PP", None::<&str>);
                    }
                    Block::Section => {
                        capture.1 = false;
                        manpage.section(&capture.0);
                        capture.0.clear();
                    }
                    Block::Subsection => {
                        capture.1 = false;
                        manpage.subsection(&capture.0);
                        capture.0.clear();
                    }
                    Block::UnnumberedList => todo!(),
                    Block::NumberedList => todo!(),
                    Block::DefinitionList => {}
                    Block::ListItem => {
                        manpage
                            .raw()
                            .control("PP", None::<&str>)
                            .strip_newlines(false);
                    }
                    Block::ListKey => {
                        manpage.raw().roff_linebreak();
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
