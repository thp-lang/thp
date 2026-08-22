use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use tempfile::TempDir;
use thp_config::{
    ByteSize, Duration, ExtensionName, Limit, ProjectConfig, RuntimeConfig, TargetName,
};

fn project(source: &str) -> TempDir {
    let root = tempfile::tempdir().expect("temporary project");
    fs::write(root.path().join("thp.toml"), source).expect("write project configuration");
    root
}

#[test]
fn empty_project_uses_built_in_defaults() {
    let root = project("");
    let profile = ProjectConfig::load(root.path())
        .expect("load")
        .resolve(None)
        .expect("resolve");
    assert_eq!(profile.runtime, RuntimeConfig::default());
    assert!(profile.extensions().is_empty());
}

#[test]
fn parses_validated_inline_configuration_without_filesystem_access() {
    let config = ProjectConfig::parse(
        "fixtures/example.phpt",
        r#"
[time]
max_execution = "2s"
"#,
    )
    .expect("parse inline configuration");
    assert_eq!(config.root(), std::path::Path::new("fixtures"));
    assert_eq!(
        config.resolve(None).expect("resolve").runtime.max_execution,
        Limit::Finite(Duration::from_seconds(2))
    );

    let error = ProjectConfig::parse(
        "fixtures/invalid.phpt",
        r#"
[time]
max_execution = "2ms"
"#,
    )
    .expect_err("invalid duration");
    assert_eq!(error.path, std::path::Path::new("fixtures/invalid.phpt"));
    assert_eq!(error.field.as_deref(), Some("time.max_execution"));
}

#[test]
fn missing_project_is_an_error_but_missing_local_is_not() {
    let root = tempfile::tempdir().expect("temporary project");
    let error = ProjectConfig::load(root.path()).expect_err("thp.toml is required");
    assert_eq!(error.path, root.path().join("thp.toml"));
    assert!(error.message.contains("required"));

    fs::write(root.path().join("thp.toml"), "").expect("write project configuration");
    ProjectConfig::load(root.path()).expect("thp.local.toml is optional");
}

#[test]
fn all_four_precedence_layers_merge_in_order() {
    let root = project(
        r#"
[memory]
limit = "100M"

[request]
post_max_size = "10M"
max_stack_depth = 600
max_open_handles = 300

[time]
max_input = "10s"
max_execution = "20s"

[targets.cli.memory]
limit = "200M"

[targets.cli.time]
max_execution = "40s"
"#,
    );
    fs::write(
        root.path().join("thp.local.toml"),
        r#"
[memory]
limit = "110M"

[time]
max_input = "11s"

[targets.cli.memory]
limit = "220M"

[targets.cli.request]
post_max_size = "22M"
max_stack_depth = 700
max_open_handles = 350
"#,
    )
    .expect("write local configuration");

    let config = ProjectConfig::load(root.path()).expect("load");
    let common = config.resolve(None).expect("resolve common");
    assert_eq!(
        common.runtime.memory_limit,
        Limit::Finite(ByteSize::from_bytes(110 * 1024 * 1024))
    );
    assert_eq!(
        common.runtime.post_max_size,
        Limit::Finite(ByteSize::from_bytes(10 * 1024 * 1024))
    );
    assert_eq!(common.runtime.max_stack_depth, Some(600));
    assert_eq!(common.runtime.max_open_handles, Some(300));
    assert_eq!(
        common.runtime.max_input,
        Limit::Finite(Duration::from_seconds(11))
    );
    assert_eq!(
        common.runtime.max_execution,
        Limit::Finite(Duration::from_seconds(20))
    );

    let cli = config.resolve(Some("cli")).expect("resolve target");
    assert_eq!(
        cli.runtime.memory_limit,
        Limit::Finite(ByteSize::from_bytes(220 * 1024 * 1024))
    );
    assert_eq!(
        cli.runtime.post_max_size,
        Limit::Finite(ByteSize::from_bytes(22 * 1024 * 1024))
    );
    assert_eq!(cli.runtime.max_stack_depth, Some(700));
    assert_eq!(cli.runtime.max_open_handles, Some(350));
    assert_eq!(
        cli.runtime.max_input,
        Limit::Finite(Duration::from_seconds(11))
    );
    assert_eq!(
        cli.runtime.max_execution,
        Limit::Finite(Duration::from_seconds(40))
    );
}

#[test]
fn undeclared_target_falls_back_to_common_profile() {
    let root = project(
        r#"
[memory]
limit = "300M"

[targets.web.memory]
limit = "400M"
"#,
    );
    let config = ProjectConfig::load(root.path()).expect("load");
    assert_eq!(
        config.resolve(Some("worker")).expect("resolve"),
        config.resolve(None).expect("resolve common")
    );
    assert_eq!(
        config
            .resolve_all()
            .expect("resolve project")
            .select(Some("worker")),
        &config.resolve(None).expect("resolve common")
    );
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct ExampleExtension {
    enabled: bool,
    labels: Vec<String>,
    nested: Nested,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Nested {
    left: i64,
    right: i64,
}

#[test]
fn extensions_merge_tables_recursively_and_replace_arrays_and_scalars() {
    let root = project(
        r#"
[extensions.example]
enabled = true
labels = ["project"]

[extensions.example.nested]
left = 1
right = 2

[targets.cli.extensions.example]
labels = ["target"]
"#,
    );
    fs::write(
        root.path().join("thp.local.toml"),
        r#"
[extensions.example]
labels = ["local"]

[extensions.example.nested]
right = 3

[targets.cli.extensions.example]
enabled = false
"#,
    )
    .expect("write local configuration");

    let config = ProjectConfig::load(root.path()).expect("load");
    let common: ExampleExtension = config
        .resolve(None)
        .expect("resolve")
        .extension("example")
        .expect("extension")
        .deserialize()
        .expect("decode extension");
    assert_eq!(
        common,
        ExampleExtension {
            enabled: true,
            labels: vec!["local".to_owned()],
            nested: Nested { left: 1, right: 3 },
        }
    );

    let target: ExampleExtension = config
        .resolve(Some("cli"))
        .expect("resolve")
        .extension("example")
        .expect("extension")
        .deserialize()
        .expect("decode extension");
    assert_eq!(
        target,
        ExampleExtension {
            enabled: false,
            labels: vec!["target".to_owned()],
            nested: Nested { left: 1, right: 3 },
        }
    );
}

#[test]
fn accepts_zero_and_unlimited_for_every_limit() {
    let root = project(
        r#"
[memory]
limit = "0M"

[request]
post_max_size = "unlimited"
max_stack_depth = 0
max_open_handles = 0

[time]
max_input = "0s"
max_execution = "unlimited"
"#,
    );
    let runtime = ProjectConfig::load(root.path())
        .expect("load")
        .resolve(None)
        .expect("resolve")
        .runtime;
    assert!(runtime.memory_limit.is_unlimited());
    assert!(runtime.post_max_size.is_unlimited());
    assert_eq!(runtime.max_stack_depth, None);
    assert_eq!(runtime.max_open_handles, None);
    assert!(runtime.max_input.is_unlimited());
    assert!(runtime.max_execution.is_unlimited());
}

#[test]
fn rejects_unknown_core_tables_and_fields() {
    for source in [
        "[php]\nmemory_limit = \"1M\"\n",
        "[memory]\nunknown = \"1M\"\n",
        "[targets.cli]\nunknown = 1\n",
    ] {
        let root = project(source);
        let error = ProjectConfig::load(root.path()).expect_err(source);
        assert!(error.message.contains("unknown field"), "{}", error.message);
        assert!(error.location.is_some());
    }
}

#[test]
fn rejects_malformed_overflowing_and_non_string_values_with_context() {
    for (field, source) in [
        ("memory.limit", "[memory]\nlimit = \"1.5M\"\n"),
        (
            "request.post_max_size",
            "[request]\npost_max_size = \"18446744073709551615G\"\n",
        ),
        ("time.max_input", "[time]\nmax_input = \"1h30m\"\n"),
        (
            "targets.cli.time.max_execution",
            "[targets.cli.time]\nmax_execution = \"-1s\"\n",
        ),
    ] {
        let root = project(source);
        let error = ProjectConfig::load(root.path()).expect_err(source);
        assert_eq!(error.field.as_deref(), Some(field));
        let location = error.location.expect("semantic error location");
        assert!(location.line >= 1);
        assert!(location.column >= 1);
    }

    let root = project("[memory]\nlimit = 12\n");
    let error = ProjectConfig::load(root.path()).expect_err("must reject non-string");
    assert!(error.message.contains("invalid type"));
}

#[test]
fn validates_target_and_extension_identifiers_and_reserved_name() {
    for source in [
        "[targets.Bad]\n",
        "[targets.default]\n",
        "[targets.\"1cli\"]\n",
        "[extensions.Bad]\nvalue = true\n",
        "[extensions.default]\nvalue = true\n",
    ] {
        let root = project(source);
        let error = ProjectConfig::load(root.path()).expect_err(source);
        assert!(error.field.is_some());
    }
    assert!(TargetName::new("cli-worker_2").is_ok());
    assert!(ExtensionName::new("cache-2").is_ok());
}

#[test]
fn parses_string_and_ordered_list_autoload_mappings() {
    let config = ProjectConfig::parse(
        "thp.toml",
        r#"
[autoload]
"App\\" = "src/"
"Vendor\\Package\\" = ["vendor/package/src/", "../shared/"]
"#,
    )
    .unwrap();
    assert_eq!(config.autoload()["App\\"], [PathBuf::from("src/")]);
    assert_eq!(
        config.autoload()["Vendor\\Package\\"],
        [
            PathBuf::from("vendor/package/src/"),
            PathBuf::from("../shared/")
        ]
    );
}

#[test]
fn rejects_invalid_autoload_prefixes_and_empty_directory_lists() {
    for (source, message) in [
        ("[autoload]\nApp = \"src\"\n", "must end with"),
        (
            "[autoload]\n\"App\\\\Bad-Name\\\\\" = \"src\"\n",
            "valid case-sensitive name segments",
        ),
        (
            "[autoload]\n\"App\\\\\" = []\n",
            "at least one non-empty directory",
        ),
    ] {
        let error = ProjectConfig::parse("thp.toml", source).unwrap_err();
        assert!(error.message.contains(message), "{}", error.message);
    }
}

#[test]
fn extension_entries_must_be_tables() {
    let root = project("[extensions]\nexample = \"not a table\"\n");
    let error = ProjectConfig::load(root.path()).expect_err("scalar extension");
    assert_eq!(error.field.as_deref(), Some("extensions.example"));
    assert!(error.message.contains("table"));
}

#[test]
fn syntax_diagnostics_include_path_and_source_position() {
    let root = project("[memory\nlimit = \"1M\"\n");
    let error = ProjectConfig::load(root.path()).expect_err("invalid TOML");
    assert_eq!(error.path, root.path().join("thp.toml"));
    let location = error.location.expect("syntax location");
    assert_eq!(location.line, 1);
    assert!(location.column >= 1);
}

#[test]
fn local_diagnostics_identify_the_local_source_and_target_field() {
    let root = project("");
    fs::write(
        root.path().join("thp.local.toml"),
        "[targets.cli.time]\nmax_execution = \"1h30m\"\n",
    )
    .expect("write local configuration");
    let error = ProjectConfig::load(root.path()).expect_err("invalid local value");
    assert_eq!(error.path, root.path().join("thp.local.toml"));
    assert_eq!(
        error.field.as_deref(),
        Some("targets.cli.time.max_execution")
    );
    assert_eq!(error.location.expect("location").line, 2);
}
