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

#[test]
fn lock_command_and_package_entrypoint_use_discovered_packages() {
    let directory = tempfile::tempdir().expect("project");
    let package = directory.path().join("vendor/acme/tool");
    std::fs::create_dir_all(package.join("src")).expect("package");
    std::fs::write(
        directory.path().join("thp.toml"),
        "[autoload]\npackages = \"vendor/\"\n",
    )
    .expect("project manifest");
    std::fs::write(
        package.join("thp.toml"),
        "[autoload]\n\"Acme\\\\Tool\\\\\" = \"src/\"\n",
    )
    .expect("package manifest");
    std::fs::write(
        package.join("src/Message.thp"),
        "<?thp\nnamespace Acme\\Tool;\nfunction message(): string { return \"package\"; }\n",
    )
    .expect("package source");
    std::fs::write(
        package.join("bin.thp"),
        "<?thp\nuse function Acme\\Tool\\message;\necho message();\n",
    )
    .expect("package entry");
    std::fs::create_dir_all(directory.path().join("public")).expect("public");
    std::fs::create_dir_all(directory.path().join("bin")).expect("bin");
    std::fs::write(
        directory.path().join("public/index.thp"),
        "<?thp\necho \"web\";\n",
    )
    .expect("web entry");
    std::fs::write(
        directory.path().join("bin/console.thp"),
        "<?thp\necho \"cli\";\n",
    )
    .expect("CLI entry");

    let root = directory.path().to_str().expect("UTF-8 path");
    let lock = thp(&["lock", &format!("--project={root}")]);
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    assert!(directory.path().join("thp.lock").is_file());
    let run = thp(&[
        "run",
        "--engine=vm",
        &format!("--project={root}"),
        "vendor/acme/tool/bin.thp",
    ]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "package");
    for (entry, expected) in [("public/index.thp", "web"), ("bin/console.thp", "cli")] {
        let run = thp(&["run", "--engine=vm", &format!("--project={root}"), entry]);
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    }

    std::fs::write(package.join("thp.toml"), "# stale\n").expect("change manifest");
    let stale = thp(&["check", &format!("--project={root}"), "public/index.thp"]);
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("stale"));
}
