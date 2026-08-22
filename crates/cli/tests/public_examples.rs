use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("CLI crate is inside the workspace")
        .to_owned()
}

fn thp(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_thp"))
        .current_dir(workspace_root())
        .args(arguments)
        .output()
        .expect("run thp")
}

fn assert_example(arguments: &[&str], expected: &str) {
    let output = thp(arguments);
    assert!(
        output.status.success(),
        "thp {arguments:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn all_public_examples_have_deterministic_output() {
    assert_example(
        &["run", "examples/hello.thp"],
        "Hello, world!\nHello, THP!\n",
    );
    assert_example(&["run", "examples/jit.thp"], "42\n");
    assert_example(&["run", "examples/objects.thp"], "closed demo\ncaught\n");
    assert_example(
        &["run", "--project=examples/project", "main.thp"],
        "Hello, Ada!\nHello, Linus!\nerror: name must not be empty\n",
    );
}

#[test]
fn check_reports_an_expected_diagnostic_failure() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let source = directory.path().join("invalid.thp");
    std::fs::write(&source, "<?thp\n$value: int = \"wrong\";\n").expect("write invalid source");

    let output = Command::new(env!("CARGO_BIN_EXE_thp"))
        .args(["check", source.to_str().expect("UTF-8 temporary path")])
        .output()
        .expect("run thp check");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error[T"),
        "unexpected diagnostic: {stderr}"
    );
    assert!(stderr.contains("expected `int`, found `string`"));
}
