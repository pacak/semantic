//! Semantic markup layer

use crate::{
    monoid::FreeMonoid,
    roff::{Apostrophes, Font},
    shared::{Section, Style},
};
use std::ops::{Add, AddAssign};

/// Semantic document that can be rendered as markdown or man page
#[derive(Debug, Clone, Default)]
pub struct Doc(FreeMonoid<Sem>);

impl AddAssign<&Self> for Doc {
    fn add_assign(&mut self, rhs: &Self) {
        self.0 += &rhs.0;
    }
}

impl Add<&Self> for Doc {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Doc(self.0 + &rhs.0)
    }
}

impl<'a> Extend<&'a Doc> for Doc {
    fn extend<I: IntoIterator<Item = &'a Doc>>(&mut self, iter: I) {
        for i in iter {
            *self += i;
        }
    }
}

impl Doc {
    /// Insert document section name
    ///
    /// ```rust
    /// # use ::roff::*;
    /// let mut doc = Doc::default();
    /// doc.section("Hello")
    ///     .paragraph("Some plain text.");
    /// ```
    ///
    /// When rendered as manpage it will use all caps
    pub fn section(&mut self, name: &str) -> &mut Self {
        self.push(&Scoped(LogicalBlock::Section, text(name)))
    }

    /// Insert document subsection name
    ///
    /// ```rust
    /// # use ::roff::*;
    /// let mut doc = Doc::default();
    /// doc.subsection("Hello")
    ///     .paragraph("Some plain text.");
    /// ```
    pub fn subsection(&mut self, name: &str) -> &mut Self {
        self.push(&Scoped(LogicalBlock::Subsection, text(name)))
    }

    /// Add a paragraph of text
    ///
    /// Paragraphs will be logically separated from each other by empty lines or indentation.
    /// Contents of a paragraph can be
    ///
    /// ```rust
    /// # use ::roff::*;
    /// ```
    pub fn paragraph<S>(&mut self, text: S) -> &mut Self
    where
        S: Write,
    {
        self.push(&Scoped(LogicalBlock::Paragraph, text))
    }

    /// Add a preformatted block of text
    ///
    /// Paragraphs will be logically separated from each other by empty lines or indentation
    pub fn pre<S>(&mut self, text: S) -> &mut Self
    where
        S: Write,
    {
        self.push(&Scoped(LogicalBlock::Pre, text))
    }

    /// Insert a numbered list
    ///
    /// Items should contain one or more [`item`](Self::item) fragments
    pub fn nlist<S>(&mut self, items: S) -> &mut Self
    where
        S: Write,
    {
        self.push(&Scoped(LogicalBlock::NumberedList, items))
    }

    /// Insert an unnumbered list
    ///
    /// Items should contain one or more [`item`](Self::item) fragments
    pub fn ulist<S>(&mut self, items: S) -> &mut Self
    where
        S: Write,
    {
        self.push(&Scoped(LogicalBlock::UnnumberedList, items))
    }

    /// Insert a definition list
    ///
    /// Items should contain a combination of [`item`](Self::item), [`term`](Self::term) or
    /// [`definition`](Self::definition) fragments.
    pub fn dlist<S>(&mut self, items: S) -> &mut Self
    where
        S: Write,
    {
        self.push(&Scoped(LogicalBlock::DefinitionList, items))
    }

    /// Insert a list item
    ///
    /// Contents should be text level fragments, for [`definition
    /// lists`](Doc::dlist) this will be used in the term body field.
    pub fn item<S>(&mut self, item: S) -> &mut Self
    where
        S: Write,
    {
        self.push(&Scoped(LogicalBlock::ListItem, item))
    }

    /// Insert a term into a definition list
    ///
    /// Contents should be a text level fragments
    pub fn term<T>(&mut self, term: T) -> &mut Self
    where
        T: Write,
    {
        self.push(&Scoped(LogicalBlock::ListKey, term))
    }

    /// Insert a definition into a definition list
    ///
    /// Combines both [`item`](Self::item) and [`term`](Self::term)
    pub fn definition<T, D>(&mut self, term: T, definition: D) -> &mut Self
    where
        T: Write,
        D: Write,
    {
        self.push(&Scoped(LogicalBlock::ListKey, term));
        self.push(&Scoped(LogicalBlock::ListItem, definition));
        self
    }

    /// Append a semantic fragment to a document
    ///
    /// `push` consumes semantic fragment, if you only have a referece to it you
    /// can append it using [`Write`] trait directly:
    ///
    /// ```rust
    /// # let mut doc = Doc::default();
    /// let fragment = literal("cauwugo");
    /// // append a fragment without consuming it
    /// fragment.write(&mut doc);
    /// // append a fragment without consuming it
    /// doc.push(fragment);
    /// ```
    #[inline(always)]
    //
    pub fn push<S>(&mut self, text: &S) -> &mut Self
    where
        S: Write,
    {
        text.write(self);
        self
    }

    /// Monospaced text fragment
    ///
    /// Can be useful to insert fixed text fragments for formatting or semantic emphasis
    pub fn mono<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(&mono(payload.as_ref()))
    }

    /// Literal text fragment
    ///
    /// This fragment represents something user needs to type literally, usually used for command names
    /// or option flag names:
    ///
    ///
    /// ```rust
    /// # use ::roff::*;
    /// let mut doc = Doc::default();
    /// doc.text("Pass ").literal("--help").text(" to print the usage");
    /// let doc = doc.render_to_markdown();
    /// let expected = "Pass <tt><b>--help</b></tt> to print the usage";
    ///
    /// assert_eq!(doc, expected);
    /// ```
    pub fn literal<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(&literal(payload.as_ref()))
    }

    /// Metavariable fragment
    ///
    /// This fragment represents something user needs to replace with a different input, usually used for
    /// argument file name placeholders:
    ///
    /// ```rust
    /// # use ::roff::*;
    /// let mut doc = Doc::default();
    /// doc.text("To save output to file: ").literal("-o").mono(" ").metavar("FILE");
    /// let doc = doc.render_to_markdown();
    /// let expected = "To save output to file: <tt><b>-o</b> <i>FILE</i></tt>";
    ///
    /// assert_eq!(doc, expected);
    /// ```
    pub fn metavar<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(&metavar(payload.as_ref()))
    }

    /// Plain text fragment
    ///
    /// This fragment represents usual text, newlines are going to be ignored
    ///
    /// ```rust
    /// # use ::roff::*;
    /// let mut doc = Doc::default();
    /// doc.text("To save output to file: ").literal("-o").mono(" ").metavar("FILE");
    /// let doc = doc.render_to_markdown();
    /// let expected = "To save output to file: <tt><b>-o</b> <i>FILE</i></tt>";
    ///
    /// assert_eq!(doc, expected);
    /// ```
    pub fn text<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(&text(payload.as_ref()))
    }

    /// Important text fragment
    ///
    /// This fragment is used to highlight some text
    ///
    /// ```rust
    /// # use ::roff::*;
    /// let mut doc = Doc::default();
    /// doc.text("Please ").important("do not").text(" the cat!");
    /// let doc = doc.render_to_markdown();
    /// let expected = "Please <b>do not</b> the cat!";
    ///
    /// assert_eq!(doc, expected);
    /// ```
    pub fn important<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(&important(payload.as_ref()))
    }
}

/*
/// A semantic document fragment that can be appended to [`Doc`] document
///
/// Semantic documents are designed with composing them from arbitrary typed chunks, not just
/// styled text. For example if document talks about command line option it should be possible to
/// insert this option by referring to a parser rather than by a string so documentation becomes
/// checked with a compiler
*/

/// A trait for writing an object containing text and semantic markup into a buffer
///
/// This trait is defined for several simple types as well as several collections - for as long as
/// it is also implemented for items inside the collections
///
/// ```rust
/// # use ::roff::*;
/// let mut doc = Doc::default();
/// // most methods take a single semantic object which you can create either using
/// // from other objects either using a closure or a slice as shown here
/// doc.paragraph(|doc: &mut Doc| { // <- closure, note the type signature
///     // slices are a bit easier to use than closures but values inside must have the same type
///     doc.push(&[StyledChar(Style::Literal, '-'), StyledChar(Style::Literal, 'h')]);
///     doc.push(&[text(" and "), literal("--help"), text(" prints usage")]);
/// });
/// let doc = doc.render_to_markdown();
///
/// let expected = "<p><tt><b>-h</b></tt> and <tt><b>--help</b></tt> prints usage</p>";
/// assert_eq!(doc, expected);
/// ```
pub trait Write {
    /// Append a fragment to a semantic document
    fn write(&self, to: &mut Doc);
}

impl<F: Fn(&mut Doc)> Write for F {
    fn write(&self, to: &mut Doc) {
        (self)(to)
    }
}

impl<const N: usize, S> Write for &[S; N]
where
    S: Write,
{
    fn write(&self, to: &mut Doc) {
        self.as_slice().write(to);
    }
}

impl<S> Write for &[S]
where
    S: Write,
{
    fn write(&self, to: &mut Doc) {
        for item in self.iter() {
            item.write(to)
        }
    }
}

impl Write for &str {
    fn write(&self, to: &mut Doc) {
        to.0.push_str(Sem::Style(Style::Text), self);
    }
}

impl Write for char {
    fn write(&self, to: &mut Doc) {
        to.0.push(Sem::Style(Style::Text), *self);
    }
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

    /// A preformatted block of text
    Pre,

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

impl<S> Write for (Style, S)
where
    S: AsRef<str>,
{
    fn write(&self, to: &mut Doc) {
        to.0.squash = true;
        to.0.push_str(Sem::Style(self.0), self.1.as_ref());
    }
}
#[derive(Debug, Copy, Clone)]
/// A struct used to append a single character with attached style
///
/// ```rust
/// # use ::roff::*;
/// let mut doc = Doc::default();
/// doc.push(&[
///     StyledChar(Style::Literal, '-'),
///     StyledChar(Style::Literal, 'h'),
/// ]);
/// assert_eq!(doc.render_to_markdown(), "<tt><b>-h</b></tt>");
/// ```
pub struct StyledChar(pub Style, pub char);

impl Write for StyledChar {
    fn write(&self, to: &mut Doc) {
        to.0.squash = true;
        to.0.push(Sem::Style(self.0), self.1);
    }
}

/// <tt><b>Literal</b></tt> text fragment
///
/// This fragment represents something user needs to type literally, usually used for command names
/// or option flag names:
///
/// ```rust
/// # use ::roff::*;
/// let mut doc = Doc::default();
/// doc.push(&[text("Pass "), literal("--help"), text(" to print the usage")]);
/// let doc = doc.render_to_markdown();
/// let expected = "Pass <tt><b>--help</b></tt> to print the usage";
///
/// assert_eq!(doc, expected);
/// ```
pub fn literal<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Literal, payload)
}

/// <tt><i>Metavariable</i></tt> text fragment
///
/// This fragment represents something user needs to replace with a different input, usually used for
/// argument file name placeholders:
///
/// ```rust
/// # use ::roff::*;
/// let mut doc = Doc::default();
/// doc.push(&[text("To save output to file: "), literal("-o"), mono(" "), metavar("FILE")]);
/// let doc = doc.render_to_markdown();
/// let expected = "To save output to file: <tt><b>-o</b> <i>FILE</i></tt>";
///
/// assert_eq!(doc, expected);
/// ```
pub fn metavar<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Metavar, payload)
}

/// Plain text fragment
///
/// This fragment should be used for any boring plaintext fragments:
///
/// ```rust
/// # use ::roff::*;
/// let mut doc = Doc::default();
/// doc.push(&[text("To save output to file: "), literal("-o"), mono(" "), metavar("FILE")]);
/// let doc = doc.render_to_markdown();
/// let expected = "To save output to file: <tt><b>-o</b> <i>FILE</i></tt>";
///
/// assert_eq!(doc, expected);
/// ```
pub fn text<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Text, payload)
}

/// <tt>Monospaced</tt> text fragment
///
/// Can be useful to insert fixed text fragments for formatting or semantic emphasis
pub fn mono<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Mono, payload)
}

/// <b>Important</b> text fragment
///
/// Can be useful for any text that should attract users's attention
pub fn important<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Important, payload)
}

struct Scoped<T>(pub LogicalBlock, pub T);
impl<S> Write for Scoped<S>
where
    S: Write,
{
    fn write(&self, to: &mut Doc) {
        to.0.squash = false;
        to.0.push_str(Sem::BlockStart(self.0), "");
        self.1.write(to);
        to.0.squash = false;
        to.0.push_str(Sem::BlockEnd(self.0), "");
    }
}

// -------------------------------------------------------------

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

#[derive(Copy, Clone, Default)]
struct Styles {
    mono: bool,
    bold: bool,
    italic: bool,
}
impl From<Style> for Styles {
    fn from(f: Style) -> Self {
        match f {
            Style::Literal => Styles {
                bold: true,
                mono: true,
                italic: false,
            },
            Style::Metavar => Styles {
                bold: false,
                mono: true,
                italic: true,
            },
            Style::Mono => Styles {
                bold: false,
                mono: true,
                italic: false,
            },
            Style::Text => Styles {
                bold: false,
                mono: false,
                italic: false,
            },
            Style::Important => Styles {
                bold: true,
                mono: false,
                italic: false,
            },
        }
    }
}

impl Doc {
    /// Render semantic document into markdown
    // not quite markdown but encasing things in html block items makes it so
    // rustdoc avoids replacing -- to unicode dash - a nice side effect to have
    #[must_use]
    #[allow(clippy::too_many_lines)] // not that many
    pub fn render_to_markdown(&self) -> String {
        let mut res = String::new();
        let mut cur_style = Styles::default();

        fn change_style(res: &mut String, cur: &mut Styles, new: Styles) {
            if cur.italic && !new.italic {
                res.push_str("</i>")
            }
            if cur.bold && !new.bold {
                res.push_str("</b>")
            }
            if cur.mono && !new.mono {
                res.push_str("</tt>")
            }
            if !cur.mono && new.mono {
                res.push_str("<tt>")
            }
            if !cur.bold && new.bold {
                res.push_str("<b>")
            }
            if !cur.italic && new.italic {
                res.push_str("<i>")
            }
            *cur = new;
        }

        // Items inside definition lists are encased in <dd> instead of <li>
        let mut is_dlist = false;
        for (meta, payload) in &self.0 {
            if !matches!(meta, Sem::Style(_)) {
                change_style(&mut res, &mut cur_style, Styles::default());
            }
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
                    LogicalBlock::Pre => {
                        blank_line(&mut res);
                        res.push_str("<pre>");
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
                    LogicalBlock::Pre => res.push_str("</pre>"),
                    LogicalBlock::Section | LogicalBlock::Subsection => {
                        blank_line(&mut res);
                    }
                },
                Sem::Style(style) => {
                    change_style(&mut res, &mut cur_style, Styles::from(*style));
                    res.push_str(payload);
                }
            }
        }
        change_style(&mut res, &mut cur_style, Styles::default());
        res
    }

    /// Render semantic document into a manpage
    ///
    /// Create a new manpage with given `title` in a given `section`
    ///
    /// `extra` can contain up to 3 items to populate header and footer:
    ///
    /// ```text
    /// [footer-middle [footer-inside [header-middle]]]
    /// ```
    /// where usual meanings are
    /// - *footer-middle* - free form date when the application was last updated
    /// - *footer-inside* - if a program is a part of some project or a suite - it goes here
    /// - *header-middle* - fancier, human readlable application name
    ///
    /// `extra` values should not be empty, but it's OK to have less than 3 items
    #[must_use]
    pub fn render_to_manpage(&self, title: &str, section: Section, extra: &[&str]) -> String {
        let mut roff = crate::roff::Roff::default();

        roff.control(
            "TH",
            [title, section.as_str()].iter().chain(extra.iter().take(3)),
        );

        // sections and subsections are implemented with .SH and .SS
        // control messages and it is easier to provide them right away
        // We also strip styling from them and change sections to all caps
        let mut capture = (String::new(), false);
        #[derive(Clone, Copy)]
        enum ListKind {
            Def,
            Ol(usize),
            Ul,
        }
        let mut kind = ListKind::Def;
        for (meta, payload) in &self.0 {
            match meta {
                Sem::BlockStart(b) => match b {
                    LogicalBlock::Section | LogicalBlock::Subsection => {
                        capture.1 = true;
                    }
                    LogicalBlock::Pre => {
                        // .nf - turn off fill mode
                        roff.control0("PP").control0("nf").strip_newlines(false);
                    }
                    LogicalBlock::Paragraph => {
                        roff.control0("PP");
                    }
                    LogicalBlock::UnnumberedList => {
                        kind = ListKind::Ul;
                    }
                    LogicalBlock::NumberedList => {
                        kind = ListKind::Ol(1);
                    }
                    LogicalBlock::DefinitionList => {
                        kind = ListKind::Def;
                    }
                    LogicalBlock::ListItem => match &mut kind {
                        ListKind::Def => {
                            //roff.control0("IP");
                        }
                        ListKind::Ol(ix) => {
                            roff.text([(Font::Roman, format!("{}. ", ix))]);
                            *ix += 1;
                        }
                        ListKind::Ul => {
                            roff.text([(Font::Roman, "* ")]);
                        }
                    },
                    LogicalBlock::ListKey => {
                        roff.control0("TP").strip_newlines(true);
                    }
                },
                Sem::BlockEnd(b) => match b {
                    LogicalBlock::Paragraph => {}
                    LogicalBlock::Pre => {
                        // .fi - restore fill mode
                        roff.control0("fi").strip_newlines(true);
                    }

                    LogicalBlock::Section => {
                        capture.1 = false;
                        roff.control("SH", [capture.0.to_uppercase()]);
                        capture.0.clear();
                    }
                    LogicalBlock::Subsection => {
                        capture.1 = false;
                        roff.control("SS", [&capture.0]);
                        capture.0.clear();
                    }
                    LogicalBlock::UnnumberedList | LogicalBlock::NumberedList => {

                        //roff.control0("RE");
                    }
                    LogicalBlock::DefinitionList => {}
                    LogicalBlock::ListItem => {
                        roff.control0("PP").strip_newlines(false);
                    }
                    LogicalBlock::ListKey => {
                        roff.roff_linebreak().strip_newlines(false);
                    }
                },
                Sem::Style(_) if capture.1 => {
                    capture.0.push_str(payload);
                }
                Sem::Style(s) => {
                    roff.text([(s.font(), payload)]);
                }
            }
        }

        roff.render(Apostrophes::Handle)
    }
}
