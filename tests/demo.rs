use ::roff::write_updated;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn file(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join(name)
}

#[test]
fn semantic_to_markdown_and_man() {
    use ::roff::man::Manpage;
    use ::roff::semantic::*;
    let mut doc = Semantic::default();
    doc.section("Description");
    doc.paragraph([text("Pass "), literal("--help"), text(" for info.")]);
    doc.section("Options");
    doc.definition_list([write_with(|doc| {
        doc.definition(
            [literal("-v"), mono(" "), literal("--verbose")],
            text("Use verbose output"),
        )
        .definition(literal("--help"), text("Print usage"))
        .definition(literal("--version"), text("Print version"));
    })]);
    doc.paragraph(text("Exit code:\n 0: if OK\n 1: if not OK"));

    let expected = "\
# Description

Pass <tt><b>\\-\\-help</b></tt> for info.

# Options

<dl>
<dt><tt><b>-v</b></tt><tt> </tt><tt><b>--verbose</b></tt></dt>
<dd>Use verbose output</dd>
<dt><tt><b>--help</b></tt></dt>
<dd>Print usage</dd>
<dt><tt><b>--version</b></tt></dt>
<dd>Print version</dd></dl>

Exit code:
 0: if OK
 1: if not OK";
    assert_eq!(doc.render_to_markdown(), expected);

    let man = Manpage::new("SIMPLE", Section::General, &[]);
    let x = doc.render_to_manpage(man);
    let sample = file("sample.1");
    let changed = write_updated(&sample, x.as_bytes()).unwrap();
    assert!(
        !changed,
        "Changes are detected to generated {:?} file",
        sample
    );
}

#[test]
fn rendering_to_file_works() {
    use ::roff::man::{Manpage, Section, Style::*};
    let page = Manpage::new("CORRUPT", Section::General, &[])
        .section("NAME")
        .paragraph([(Text, "corrupt - modify files by randomly changing bits")])
        .section("SYNOPSIS")
        .paragraph([
            (Literal, "corrupt"),
            (Text, " ["),
            (Literal, "-n"),
            (Text, " "),
            (Metavar, "BITS"),
            (Text, "] ["),
            (Literal, "--bits"),
            (Text, " "),
            (Metavar, "BITS"),
            (Text, "] "),
            (Metavar, "file"),
            (Text, "..."),
        ])
        .section("DESCRIPTION")
        .paragraph([
            (Literal, "corrupt"),
            (Text, " modifies files by toggling a randomly chosen bit."),
        ])
        .section("OPTIONS")
        .label(
            None,
            [
                (Literal, "-n"),
                (Text, ", "),
                (Literal, "--bits"),
                (Text, "="),
                (Metavar, "BITS"),
            ],
        )
        .paragraph([
            (Text, "Set the number of bits to modify. "),
            (Text, "Default is one bit."),
        ])
        .render();

    let demo = file("demo.troff");
    let changed = write_updated(&demo, page.as_bytes()).unwrap();
    assert!(!changed, "Changes detected to generated {:?} file", demo);
}

#[cfg(unix)]
#[test]
fn rendered_file_makes_sense_to_troff() {
    let expected = file("demo.expected");
    let demo = file("demo.troff");

    let child = Command::new("troff")
        .args(["-a", "-mman"])
        .arg(demo)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let out = child.wait_with_output().expect("Child process failed");
    assert!(out.status.success());

    let output = String::from_utf8_lossy(&out.stdout);
    let changed = write_updated(&expected, output.as_bytes()).unwrap();
    assert!(
        !changed,
        "Changes detected to generated {:?} file",
        expected
    );
}
