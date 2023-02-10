//! Semantic markup layer

use crate::{
    monoid::FreeMonoid,
    roff::Apostrophes,
    shared::{Section, Style},
};
use std::ops::{Add, AddAssign};

/// Semantic document that can be rendered
///
/// See [`module`](crate::semantic) documentation for more info
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
    /// # use roff::*;
    /// let mut doc = Semantic::new();
    /// doc.section("Hello")
    ///     .paragraph("Some plain text.");
    /// ```
    ///
    /// When rendered as manpage it will use all caps
    pub fn section(&mut self, name: &str) -> &mut Self {
        self.push(Scoped(LogicalBlock::Section, text(name)))
    }

    /// Insert document subsection name
    ///
    /// ```rust
    /// # use roff::*;
    /// let mut doc = Doc::new();
    /// doc.subsection("Hello")
    ///     .paragraph("Some plain text.");
    /// ```
    pub fn subsection(&mut self, name: &str) -> &mut Self {
        self.push(Scoped(LogicalBlock::Subsection, text(name)))
    }

    /// Add a paragraph of text
    ///
    /// Paragraphs will be logically separated from each other by empty lines or indentation.
    /// Contents of a paragraph can be
    ///
    /// ```rust
    /// # use roff::*;
    /// ```
    pub fn paragraph<S>(&mut self, text: S) -> &mut Self
    where
        S: Write,
    {
        self.push(Scoped(LogicalBlock::Paragraph, text))
    }

    /// Add a preformatted block of text
    ///
    /// Paragraphs will be logically separated from each other by empty lines or indentation
    pub fn pre<S>(&mut self, text: S) -> &mut Self
    where
        S: Write,
    {
        self.push(Scoped(LogicalBlock::Pre, text))
    }

    /// Insert a numbered list
    ///
    /// Items should contain one or more [`item`](Self::item) fragments
    pub fn nlist<S>(&mut self, items: S) -> &mut Self
    where
        S: Write,
    {
        self.push(Scoped(LogicalBlock::NumberedList, items))
    }

    /// Insert an unnumbered list
    ///
    /// Items should contain one or more [`item`](Self::item) fragments
    pub fn ulist<S>(&mut self, items: S) -> &mut Self
    where
        S: Write,
    {
        self.push(Scoped(LogicalBlock::UnnumberedList, items))
    }

    /// Insert a definition list
    ///
    /// Items should contain a combination of [`item`](Self::item), [`term`](Self::term) or
    /// [`definition`](Self::definition) fragments.
    pub fn dlist<S>(&mut self, items: S) -> &mut Self
    where
        S: Write,
    {
        self.push(Scoped(LogicalBlock::DefinitionList, items))
    }

    /// Insert a list item
    ///
    /// Contents should be text level fragments, for [`definition
    /// lists`](Semantic::dlist) this will be used in the term body field.
    pub fn item<S>(&mut self, item: S) -> &mut Self
    where
        S: Write,
    {
        self.push(Scoped(LogicalBlock::ListItem, item))
    }

    /// Insert a term into a definition list
    ///
    /// Contents should be a text level fragments
    pub fn term<T>(&mut self, term: T) -> &mut Self
    where
        T: Write,
    {
        self.push(Scoped(LogicalBlock::ListKey, term))
    }

    /// Insert a definition into a definition list
    ///
    /// Combines both [`item`](Self::item) and [`term`](Self::term)
    pub fn definition<T, D>(&mut self, term: T, definition: D) -> &mut Self
    where
        T: Write,
        D: Write,
    {
        self.push(Scoped(LogicalBlock::ListKey, term));
        self.push(Scoped(LogicalBlock::ListItem, definition));
        self
    }

    /// Append
    #[inline(always)]
    pub fn push<S>(&mut self, text: S) -> &mut Self
    where
        S: Write,
    {
        text.write(self);
        self
    }

    /// Monospaced text semantic fragment
    ///
    /// Can be useful to insert fixed text fragments for formatting or semantic emphasis
    pub fn mono<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(mono(payload.as_ref()))
    }

    /// Literal semantic fragment
    ///
    /// This fragment represents something user needs to type literally, usually used for command names
    /// or option flag names:
    ///
    ///
    /// ```rust
    /// # use roff::semantic::*;
    /// let mut doc = Semantic::default();
    /// doc.text("Pass ").literal("--help").text(" to print the usage");
    /// let doc = doc.render_to_markdown();
    /// let expected = "Pass <tt><b>\\-\\-help</b></tt> to print the usage";
    ///
    /// assert_eq!(doc, expected);
    /// ```
    pub fn literal<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(literal(payload.as_ref()))
    }

    /// Metavariable semantic fragment
    ///
    /// This fragment represents something user needs to replace with a different input, usually used for
    /// argument file name placeholders:
    ///
    /// ```rust
    /// # use roff::semantic::*;
    /// let mut doc = Semantic::default();
    /// doc.text("To save output to file: ").literal("-o").mono(" ").metavar("FILE");
    /// let doc = doc.render_to_markdown();
    /// let expected = "To save output to file: <tt><b>\\-o</b></tt><tt> </tt><tt><i>FILE</i></tt>";
    ///
    /// assert_eq!(doc, expected);
    /// ```
    pub fn metavar<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(metavar(payload.as_ref()))
    }

    /// Plain text fragment
    ///
    /// This fragment represents usual text, newlines are going to be ignored
    ///
    /// ```rust
    /// # use roff::semantic::*;
    /// let mut doc = Semantic::default();
    /// doc.text("To save output to file: ").literal("-o").mono(" ").metavar("FILE");
    /// let doc = doc.render_to_markdown();
    /// let expected = "To save output to file: <tt><b>\\-o</b></tt><tt> </tt><tt><i>FILE</i></tt>";
    ///
    /// assert_eq!(doc, expected);
    /// ```
    pub fn text<S>(&mut self, payload: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.push(text(payload.as_ref()))
    }

    /// Important text fragment
    ///
    /// This fragment is used to highlight some text
    ///
    /// ```rust
    /// # use roff::semantic::*;
    /// let mut doc = Semantic::default();
    /// doc.text("Please ").important("do not").text(" the cat!");
    /// let doc = doc.render_to_markdown();
    /// let expected = "To save output to file: <tt><b>\\-o</b></tt><tt> </tt><tt><i>FILE</i></tt>";
    ///
    /// assert_eq!(doc, expected);
    /// ```
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
///
///

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
///
pub trait Write {
    /// Append a fragment of semantic document
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
///
pub struct StyledChar(pub Style, pub char);

impl Write for StyledChar {
    fn write(&self, to: &mut Doc) {
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
pub fn literal<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Literal, payload)
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
pub fn metavar<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Metavar, payload)
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
pub fn text<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Text, payload)
}

/// Monospaced text semantic fragment
///
/// Can be useful to insert fixed text fragments for formatting or semantic emphasis
pub fn mono<T>(payload: T) -> (Style, T)
where
    T: AsRef<str>,
{
    (Style::Mono, payload)
}

/// Important text semantic fragment
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

impl Doc {
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
    pub fn render_to_manpage(&self, title: &str, section: Section, extra: &[&str]) -> String {
        let mut roff = crate::roff::Roff::default();

        roff.control(
            "TH",
            [title.as_ref(), section.as_str()]
                .iter()
                .chain(extra.iter().take(3)),
        );

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
                    LogicalBlock::Pre => {
                        // .nf - turn off fill mode
                        // .eo - turn off escape processing
                        // .fi - restore fill mode
                        // .ec - restore escape processing

                        roff.control("nf", None::<&str>) // .fi
                            // .control("eo", None::<&str>) // .ec
                            .strip_newlines(false);
                    }
                    LogicalBlock::Paragraph => {
                        roff.control("PP", None::<&str>);
                    }
                    LogicalBlock::UnnumberedList
                    | LogicalBlock::NumberedList
                    | LogicalBlock::DefinitionList => {}
                    LogicalBlock::ListItem => {
                        roff.strip_newlines(true);
                    }
                    LogicalBlock::ListKey => {
                        roff.control("TP", None::<&str>).strip_newlines(true);
                    }
                },
                Sem::BlockEnd(b) => match b {
                    LogicalBlock::Paragraph => {
                        //                        manpage.raw().strip_newlines(false);
                    }
                    LogicalBlock::Pre => {
                        roff
                            // .control("ec", None::<&str>) // .fi
                            .control("fi", None::<&str>) // .ec
                            .strip_newlines(true);
                    }

                    LogicalBlock::Section => {
                        capture.1 = false;
                        roff.control("SH", &[&capture.0.to_uppercase()]);
                        capture.0.clear();
                    }
                    LogicalBlock::Subsection => {
                        capture.1 = false;
                        roff.control("SS", &[&capture.0]);
                        capture.0.clear();
                    }
                    LogicalBlock::UnnumberedList => todo!(),
                    LogicalBlock::NumberedList => todo!(),
                    LogicalBlock::DefinitionList => {}
                    LogicalBlock::ListItem => {
                        roff.control("PP", None::<&str>).strip_newlines(false);
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
