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
        .paragraph([(Normal, "corrupt - modify files by randomly changing bits")])
        .section("SYNOPSIS")
        .paragraph([
            (Argument, "corrupt"),
            (Normal, " ["),
            (Argument, "-n"),
            (Normal, " "),
            (Metavar, "BITS"),
            (Normal, "] ["),
            (Argument, "--bits"),
            (Normal, " "),
            (Metavar, "BITS"),
            (Normal, "] "),
            (Metavar, "file"),
            (Normal, "..."),
        ])
        .section("DESCRIPTION")
        .paragraph([
            (Argument, "corrupt"),
            (Normal, " modifies files by toggling a randomly chosen bit."),
        ])
        .section("OPTIONS")
        .label(
            None,
            [
                (Argument, "-n"),
                (Normal, ", "),
                (Argument, "--bits"),
                (Normal, "="),
                (Metavar, "BITS"),
            ],
        )
        .paragraph([
            (Normal, "Set the number of bits to modify. "),
            (Normal, "Default is one bit."),
        ])
        .render();

    let demo = troff_file();
    let changed = write_updated(&page, &demo).unwrap();
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
    let changed = write_updated(&output, &expected).unwrap();
    assert!(
        !changed,
        "Changes detected to generated {:?} file",
        expected
    );
}
