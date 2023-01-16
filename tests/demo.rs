use ::roff::write_updated;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn troff_file() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("demo.troff")
}

fn expected_file() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("tests")
        .join("demo.expected")
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

    let demo = troff_file();
    let changed = write_updated(&demo, &page).unwrap();
    assert!(!changed, "Changes detected to generated {:?} file", demo);
}

#[cfg(unix)]
#[test]
fn rendered_file_makes_sense_to_troff() {
    let expected = expected_file();
    let demo = troff_file();

    let child = Command::new("troff")
        .args(["-a", "-mman"])
        .arg(demo)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let out = child.wait_with_output().expect("Child process failed");
    assert!(out.status.success());

    let output = String::from_utf8_lossy(&out.stdout);
    let changed = write_updated(&expected, &output).unwrap();
    assert!(
        !changed,
        "Changes detected to generated {:?} file",
        expected
    );
}
