use roff::{write_updated, Doc, Section};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn file(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join(name)
}

fn doc() -> Doc {
    use roff::*;
    let mut doc = Doc::default();
    doc.section("Description");
    doc.paragraph(&[text("Pass "), literal("--help"), text(" for info.")]);
    doc.section("Options");
    doc.dlist(|doc: &mut Doc| {
        doc.definition(
            &[literal("-v"), mono(" "), literal("--verbose")],
            text("Use verbose output"),
        )
        .definition(literal("--help"), text("Print usage"))
        .definition(literal("--version"), text("Print version"));
    });

    doc.pre(text("Exit code:\n 0: if OK\n 1: if not OK"));

    doc.paragraph(&[
        text("A few lines\n"),
        text("of text\n"),
        text(".can  be   here"),
    ]);

    doc
}

#[test]
fn semantic_to_markdown() {
    let doc = doc();

    let expected = "\
# Description

<p>Pass <tt><b>--help</b></tt> for info.</p>

# Options

<dl>
<dt><tt><b>-v</b></tt><tt> </tt><tt><b>--verbose</b></tt></dt>
<dd>Use verbose output</dd>
<dt><tt><b>--help</b></tt></dt>
<dd>Print usage</dd>
<dt><tt><b>--version</b></tt></dt>
<dd>Print version</dd></dl>

<pre>Exit code:
 0: if OK
 1: if not OK</pre>

<p>A few lines
of text
 can be here</p>";

    assert_eq!(doc.render_to_markdown(), expected);
}

#[test]
fn semantic_to_manpage() {
    let doc = doc().render_to_manpage("SIMPLE", Section::General, &[]);
    let sample = file("sample.1");
    let changed = write_updated(&sample, doc.as_bytes()).unwrap();
    assert!(
        !changed,
        "Changes are detected to generated {:?} file",
        sample
    );
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
