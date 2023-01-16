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
    pub(crate) fn font(&self) -> Font {
        match self {
            Style::Metavar => Font::Italic,
            Style::Literal => Font::Bold,
            Style::Text => Font::Roman,
            Style::Important => Font::BoldItalic,
            Style::Mono => Font::Mono,
        }
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
    pub(crate) fn as_str(&self) -> &str {
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
