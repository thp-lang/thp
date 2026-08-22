use std::fs;
use std::process::Command;

use thp_test::{Runner, RunnerOptions, Status};

fn write_fixture(root: &std::path::Path, name: &str, contents: &[u8]) {
    fs::write(root.join(name), contents).expect("write fixture");
}

#[test]
fn repository_phpt_specifications_pass_through_the_runner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/phpt");
    let summary = Runner::default()
        .run_paths([root.join("resources"), root.join("language")])
        .expect("run fixtures");
    assert_eq!(summary.tests, 38);
    assert_eq!(summary.passed, 38);
    assert!(summary.is_success());
}

#[test]
fn lifecycle_covers_skip_bork_cleanup_and_fail_precedence() {
    let root = tempfile::tempdir().expect("tempdir");
    write_fixture(
        root.path(),
        "01-skip.phpt",
        b"--TEST--\nskip\n--SKIPIF--\n<?thp\necho \"skip unavailable\";\n--FILE--\n<?thp\necho \"bad\";\n--EXPECT--\ngood\n",
    );
    write_fixture(
        root.path(),
        "02-invalid-skip.phpt",
        b"--TEST--\ninvalid skip\n--SKIPIF--\n<?thp\necho \"maybe\";\n--FILE--\n<?thp\n--EXPECT--\n",
    );
    write_fixture(
        root.path(),
        "02-runtime-skip.phpt",
        b"--TEST--\nfailing skip\n--SKIPIF--\n<?thp\n$value = 9223372036854775807 + 1;\n--FILE--\n<?thp\n--EXPECT--\n",
    );
    write_fixture(
        root.path(),
        "03-clean-bork.phpt",
        b"--TEST--\nclean output\n--FILE--\n<?thp\necho \"ok\";\n--CLEAN--\n<?thp\necho \"noise\";\n--EXPECT--\nok\n",
    );
    write_fixture(
        root.path(),
        "03-clean-error.phpt",
        b"--TEST--\nclean failure\n--FILE--\n<?thp\necho \"ok\";\n--CLEAN--\n<?thp\n$value = 9223372036854775807 + 1;\n--EXPECT--\nok\n",
    );
    write_fixture(
        root.path(),
        "04-fail-clean.phpt",
        b"--TEST--\nfail stays fail\n--FILE--\n<?thp\necho \"actual\";\n--CLEAN--\n<?thp\necho \"noise\";\n--EXPECT--\nexpected\n",
    );
    write_fixture(
        root.path(),
        "05-unsupported.phpt",
        b"--TEST--\nunsupported ini\n--INI--\nmemory_limit=1M\n--FILE--\n<?thp\n--EXPECT--\n",
    );

    let summary = Runner::default()
        .run_paths([root.path()])
        .expect("run lifecycle fixtures");
    assert_eq!(
        summary
            .results
            .iter()
            .map(|result| result.status)
            .collect::<Vec<_>>(),
        [
            Status::Skip,
            Status::Bork,
            Status::Bork,
            Status::Bork,
            Status::Bork,
            Status::Fail,
            Status::Skip
        ]
    );
    let failed = summary
        .results
        .iter()
        .find(|result| result.name == "fail stays fail")
        .expect("failed fixture");
    assert!(
        failed
            .details
            .as_deref()
            .is_some_and(|details| details.contains("CLEAN produced output"))
    );
}

#[test]
fn relative_io_partial_errors_config_and_instruction_limits_work() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::write(root.path().join("input.txt"), b"fixture data").expect("input");
    write_fixture(
        root.path(),
        "00-generic.phpt",
        b"--TEST--\ngeneric type syntax\n--FILE--\n<?thp\n$values: vector<int> = [41, 42];\necho $values[1] . \"\\n\";\n--EXPECT--\n42\n",
    );
    write_fixture(
        root.path(),
        "01-relative.phpt",
        b"--TEST--\nrelative io\n--FILE--\n<?thp\n$stream = Files::openRead(\"input.txt\");\necho $stream->readAll();\n--EXPECT--\nfixture data\n",
    );
    write_fixture(
        root.path(),
        "02-partial-error.phpt",
        b"--TEST--\npartial error\n--FILE--\n<?thp\necho \"before\\n\";\n$value = 9223372036854775807 + 1;\n--EXPECTF--\nbefore\n%s:%d:%d: runtime error: integer addition overflow\n",
    );
    write_fixture(
        root.path(),
        "03-extension.phpt",
        b"--TEST--\nextension config\n--CONFIG--\n[extensions.demo]\nenabled = true\n--FILE--\n<?thp\n--EXPECT--\n",
    );
    write_fixture(
        root.path(),
        "04-target.phpt",
        b"--TEST--\ntarget config\n--CONFIG--\n[targets.cli]\n--FILE--\n<?thp\n--EXPECT--\n",
    );
    write_fixture(
        root.path(),
        "05-limit.phpt",
        b"--TEST--\ninstruction limit\n--FILE--\n<?thp\nwhile (true) {}\n--EXPECTF--\nexecution exceeded the %d instruction limit\n",
    );

    let summary = Runner::new(RunnerOptions {
        max_instructions: Some(20),
    })
    .run_paths([root.path()])
    .expect("run fixtures");
    assert_eq!(
        summary
            .results
            .iter()
            .map(|result| result.status)
            .collect::<Vec<_>>(),
        [
            Status::Pass,
            Status::Pass,
            Status::Pass,
            Status::Skip,
            Status::Bork,
            Status::Pass
        ],
        "{summary:#?}"
    );
}

#[test]
fn external_program_must_be_contained_and_utf8() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::write(root.path().join("program.thp"), "<?thp\necho \"ok\";").expect("program");
    fs::write(root.path().join("binary.thp"), b"\xff").expect("binary");
    write_fixture(
        root.path(),
        "01-contained.phpt",
        b"--TEST--\ncontained\n--FILE_EXTERNAL--\nprogram.thp\n--EXPECT--\nok\n",
    );
    write_fixture(
        root.path(),
        "02-traversal.phpt",
        b"--TEST--\ntraversal\n--FILE_EXTERNAL--\n../program.thp\n--EXPECT--\n",
    );
    write_fixture(
        root.path(),
        "03-binary.phpt",
        b"--TEST--\nbinary\n--FILE_EXTERNAL--\nbinary.thp\n--EXPECT--\n",
    );
    let summary = Runner::default()
        .run_paths([root.path()])
        .expect("run external fixtures");
    assert_eq!(
        summary
            .results
            .iter()
            .map(|result| result.status)
            .collect::<Vec<_>>(),
        [Status::Pass, Status::Bork, Status::Bork]
    );
}

#[test]
fn external_project_fixture_uses_configured_static_autoload_modules() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("src/Math")).expect("module directory");
    fs::write(
        root.path().join("src/Math/functions.thp"),
        "<?thp\nnamespace App\\Math;\nfunction square(int $x): int { return $x * $x; }\n",
    )
    .expect("module");
    fs::write(
        root.path().join("main.thp"),
        "<?thp\nuse function App\\Math\\square;\n$input = Streams::open(\"thp:/input\", OpenMode::Read);\necho square(5) . \"\";\necho $input->readAll();\n",
    )
    .expect("entry");
    write_fixture(
        root.path(),
        "modules.phpt",
        b"--TEST--\nstatic modules and request input\n--CONFIG--\n[autoload]\n\"App\\\\\" = \"src/\"\n--FILE_EXTERNAL--\nmain.thp\n--STDIN--\nbody\n--EXPECT--\n25body\n",
    );
    let summary = Runner::default()
        .run_paths([root.path().join("modules.phpt")])
        .expect("run module fixture");
    assert_eq!(summary.results[0].status, Status::Pass, "{summary:#?}");
}

#[test]
fn binary_reports_deterministically_and_uses_all_exit_classes() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::create_dir(root.path().join("nested")).expect("nested");
    write_fixture(
        &root.path().join("nested"),
        "b.phpt",
        b"--TEST--\nb\n--FILE--\n<?thp\necho \"b\";\n--EXPECT--\nb\n",
    );
    write_fixture(
        root.path(),
        "a.phpt",
        b"--TEST--\na\n--FILE--\n<?thp\necho \"a\";\n--EXPECT--\na\n",
    );
    let binary = env!("CARGO_BIN_EXE_thp-test");
    let success = Command::new(binary)
        .arg(root.path())
        .output()
        .expect("run binary");
    assert!(success.status.success());
    let stdout = String::from_utf8(success.stdout).expect("UTF-8 output");
    assert!(stdout.find("a.phpt").expect("a") < stdout.find("b.phpt").expect("b"));
    assert!(stdout.contains("Tests: 2, Pass: 2, Fail: 0, Skip: 0, Bork: 0"));

    write_fixture(
        root.path(),
        "c.phpt",
        b"--TEST--\nc\n--FILE--\n<?thp\necho \"no\";\n--EXPECT--\nyes\n",
    );
    let failure = Command::new(binary)
        .arg(root.path())
        .output()
        .expect("run failing binary");
    assert_eq!(failure.status.code(), Some(1));

    let invocation = Command::new(binary)
        .arg(root.path().join("missing"))
        .output()
        .expect("run invalid binary");
    assert_eq!(invocation.status.code(), Some(2));

    let empty = tempfile::tempdir().expect("empty directory");
    let no_tests = Command::new(binary)
        .arg(empty.path())
        .output()
        .expect("run empty selection");
    assert_eq!(no_tests.status.code(), Some(2));
}
