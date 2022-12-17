use crate::raw::Roff;

/// Mandoc document
#[derive(Debug, Clone)]
struct MDoc {
    roff: Roff,
}

impl MDoc {
    pub fn new(date: &str, title: &str, name: &str, description: &str) -> Self {
        let mut roff = Roff::default();
        roff.control("Dd", [date])
            .control("Dt", [title])
            .control("Os", None::<str>)
            .control("Nm", [name])
            .control("Nd", [description]);

        Self { roff }
    }
}

// Document preamble and NAME section macros
// Dd 	document date: $Mdocdate$ | month day, year
// Dt 	document title: TITLE section [arch]
// Os 	operating system version: [system [version]]
// Nm 	document name (one argument)
// Nd 	document description (one line)
//
//
// Sections and cross references
// Sh 	section header (one line)
// Ss 	subsection header (one line)
// Sx 	internal cross reference to a section or subsection
// Xr 	cross reference to another manual page: name section
// Tg 	tag the definition of a term (<= 1 arguments)
// Pp 	start a text paragraph (no arguments)

// Displays and lists
// Bd, Ed 	display block: -type [-offset width] [-compact]
// D1 	indented display (one line)
// Dl 	indented literal display (one line)
// Ql 	in-line literal display: ‘text’
// Bl, El 	list block: -type [-width val] [-offset val] [-compact]
// It 	list item (syntax depends on -type)
// Ta 	table cell separator in Bl -column lists
// Rs, %*, Re 	bibliographic block (references)

// Spacing control
// Pf 	prefix, no following horizontal space (one argument)
// Ns 	roman font, no preceding horizontal space (no arguments)
// Ap 	apostrophe without surrounding whitespace (no arguments)
// Sm 	switch horizontal spacing mode: [on | off]
// Bk, Ek 	keep block: -words

// Semantic markup for command line utilities
// Nm 	start a SYNOPSIS block with the name of a utility
// Fl 	command line options (flags) (>=0 arguments)
// Cm 	command modifier (>0 arguments)
// Ar 	command arguments (>=0 arguments)
// Op, Oo, Oc 	optional syntax elements (enclosure)
// Ic 	internal or interactive command (>0 arguments)
// Ev 	environmental variable (>0 arguments)
// Pa 	file system path (>=0 arguments)

// Semantic markup for function libraries
// Lb 	function library (one argument)
// In 	include file (one argument)
// Fd 	other preprocessor directive (>0 arguments)
// Ft 	function type (>0 arguments)
// Fo, Fc 	function block: funcname
// Fn 	function name: funcname [argument ...]
// Fa 	function argument (>0 arguments)
// Vt 	variable type (>0 arguments)
// Va 	variable name (>0 arguments)
// Dv 	defined variable or preprocessor constant (>0 arguments)
// Er 	error constant (>0 arguments)
// Ev 	environmental variable (>0 arguments)

// Various semantic markup
// An 	author name (>0 arguments)
// Lk 	hyperlink: uri [display_name]
// Mt 	“mailto” hyperlink: localpart@domain
// Cd 	kernel configuration declaration (>0 arguments)
// Ad 	memory address (>0 arguments)
// Ms 	mathematical symbol (>0 arguments)

// Physical markup
// Em 	italic font or underline (emphasis) (>0 arguments)
// Sy 	boldface font (symbolic) (>0 arguments)
// No 	return to roman font (normal) (>0 arguments)
// Bf, Ef 	font block: -type | Em | Li | Sy

// Physical enclosures
// Dq, Do, Dc 	enclose in typographic double quotes: “text”
// Qq, Qo, Qc 	enclose in typewriter double quotes: "text"
// Sq, So, Sc 	enclose in single quotes: ‘text’
// Pq, Po, Pc 	enclose in parentheses: (text)
// Bq, Bo, Bc 	enclose in square brackets: [text]
// Brq, Bro, Brc 	enclose in curly braces: {text}
// Aq, Ao, Ac 	enclose in angle brackets: ⟨text⟩
// Eo, Ec 	generic enclosure
//
// Text production
// Ex -std 	standard command exit values: [utility ...]
// Rv -std 	standard function return values: [function ...]
// St 	reference to a standards document (one argument)
// At 	AT&T UNIX
// Bx 	BSD
// Bsx 	BSD/OS
// Nx 	NetBSD
// Fx 	FreeBSD
// Ox 	OpenBSD
// Dx 	DragonFly
