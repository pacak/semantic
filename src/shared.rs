use crate::roff::Font;

/// Style and meaning of a particular snippet of text
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Style {
    /// Encased string that is or a part of an option name inclusive with dashes
    /// `-f` or `--foo`
    /// Something that user needs to type literally
    Literal,

    /// Metavariable part
    /// - FOO here: --foo FOO
    /// Something that user needs to replace with their own input
    Metavar,

    /// Monospaced text
    Mono,

    /// Plain text, no extra decorations
    Text,

    /// Highlighted part of a text
    Important,
}

impl Style {
    pub(crate) fn font(self) -> Font {
        match self {
            Style::Metavar => Font::Italic,
            Style::Literal => Font::Bold,
            Style::Text => Font::Roman,
            Style::Important => Font::BoldItalic,
            Style::Mono => Font::Mono,
        }
    }
}
