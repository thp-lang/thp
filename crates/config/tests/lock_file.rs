use std::fs;

use thp_config::{ByteSize, Limit, LockErrorKind, LockFile, build_lock, parse_lock};

fn project(source: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary project");
    fs::write(root.path().join("thp.toml"), source).expect("write project configuration");
    root
}

fn minimal_lock(body: &str) -> Vec<u8> {
    format!(
        "THP-LOCK 1\n\
         fingerprint {}\n\
         package-roots 0\n\
         autoload 0\n\
         profile common\n\
         memory.limit 134217728\n\
         request.post_max_size 8388608\n\
         request.max_stack_depth 512\n\
         request.max_open_handles 256\n\
         time.max_input 60\n\
         time.max_execution 30\n\
         {body}",
        "0".repeat(64)
    )
    .into_bytes()
}

#[test]
fn deterministic_lock_matches_the_version_one_snapshot() {
    let root = project("");
    let result = build_lock(root.path()).expect("build lock");
    assert!(result.changed);

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"THP-CONFIG-SOURCES\0\x02");
    hasher.update(&(8_u64).to_le_bytes());
    hasher.update(b"thp.toml");
    hasher.update(&[1]);
    hasher.update(&0_u64.to_le_bytes());
    hasher.update(&(14_u64).to_le_bytes());
    hasher.update(b"thp.local.toml");
    hasher.update(&[0]);
    let fingerprint = hasher.finalize().to_hex();
    let expected = format!(
        "THP-LOCK 1\n\
         fingerprint {fingerprint}\n\
         package-roots 0\n\
         autoload 0\n\
         profile common\n\
         memory.limit 134217728\n\
         request.post_max_size 8388608\n\
         request.max_stack_depth 512\n\
         request.max_open_handles 256\n\
         time.max_input 60\n\
         time.max_execution 30\n\
         end-profile\n\
         end-lock\n"
    );
    assert_eq!(
        fs::read_to_string(root.path().join("thp.lock")).expect("read lock"),
        expected
    );
    assert!(!build_lock(root.path()).expect("rebuild lock").changed);
}

#[test]
fn lock_round_trips_discovered_autoload_and_detects_package_changes() {
    let root = project("[autoload]\npackages = \"vendor/\"\n\"App\\\\\" = \"src/\"\n");
    let package = root.path().join("vendor/acme/tool");
    fs::create_dir_all(&package).expect("create package");
    fs::write(
        package.join("thp.toml"),
        "[autoload]\n\"Acme\\\\Tool\\\\\" = [\"src/\", \"generated/\"]\n",
    )
    .expect("write package manifest");

    build_lock(root.path()).expect("build lock");
    let bytes = fs::read(root.path().join("thp.lock")).expect("read lock");
    let parsed = parse_lock(&bytes).expect("parse lock");
    assert_eq!(parsed.package_roots, ["vendor/"]);
    assert_eq!(parsed.autoload[0].prefix, "Acme\\Tool\\");
    assert_eq!(
        parsed.autoload[0].directories,
        ["vendor/acme/tool/src/", "vendor/acme/tool/generated/"]
    );
    assert_eq!(
        LockFile::load(root.path())
            .expect("load")
            .project()
            .autoload["App\\"],
        [std::path::PathBuf::from("src/")]
    );

    fs::write(
        package.join("thp.toml"),
        "[autoload]\n\"Changed\\\\\" = \"src/\"\n",
    )
    .expect("edit manifest");
    assert_eq!(
        LockFile::load(root.path()).unwrap_err().kind,
        LockErrorKind::Stale
    );
    build_lock(root.path()).expect("rebuild");
    fs::create_dir_all(root.path().join("vendor/other/new")).expect("add package");
    fs::write(root.path().join("vendor/other/new/thp.toml"), "").expect("manifest");
    assert_eq!(
        LockFile::load(root.path()).unwrap_err().kind,
        LockErrorKind::Stale
    );
    build_lock(root.path()).expect("rebuild");
    fs::remove_dir_all(root.path().join("vendor/other/new")).expect("remove package");
    assert_eq!(
        LockFile::load(root.path()).unwrap_err().kind,
        LockErrorKind::Stale
    );
}

#[test]
fn deleting_an_empty_package_root_invalidates_the_lock() {
    let root = project("[autoload]\npackages = \"vendor/\"\n");
    fs::create_dir(root.path().join("vendor")).expect("create package root");
    build_lock(root.path()).expect("build lock");

    fs::remove_dir(root.path().join("vendor")).expect("remove package root");
    let error = LockFile::load(root.path()).expect_err("missing package root must invalidate lock");
    assert!(
        matches!(error.kind, LockErrorKind::Source | LockErrorKind::Stale),
        "{error}"
    );
}

#[test]
fn old_version_one_layout_requires_regeneration() {
    let old = format!(
        "THP-LOCK 1\nfingerprint {}\nprofile common\n",
        "0".repeat(64)
    );
    let error = parse_lock(old.as_bytes()).expect_err("old layout");
    assert_eq!(error.kind, LockErrorKind::Corrupt);
    assert!(error.message.contains("thp lock"));
}

#[test]
fn lock_load_checks_freshness_and_target_fallback() {
    let root = project(
        r#"
[memory]
limit = "100M"

[targets.cli.memory]
limit = "200M"
"#,
    );
    build_lock(root.path()).expect("build lock");
    let lock = LockFile::load(root.path()).expect("load fresh lock");
    assert_eq!(
        lock.select(Some("cli")).runtime.memory_limit,
        Limit::Finite(ByteSize::from_bytes(200 * 1024 * 1024))
    );
    assert_eq!(
        lock.select(Some("undeclared")).runtime.memory_limit,
        Limit::Finite(ByteSize::from_bytes(100 * 1024 * 1024))
    );

    fs::write(root.path().join("thp.toml"), "[memory]\nlimit = \"101M\"\n").expect("change source");
    assert_eq!(
        LockFile::load(root.path()).expect_err("stale lock").kind,
        LockErrorKind::Stale
    );
}

#[test]
fn local_file_appearance_and_disappearance_make_lock_stale() {
    let root = project("");
    build_lock(root.path()).expect("build lock");
    fs::write(root.path().join("thp.local.toml"), "").expect("add local source");
    assert_eq!(
        LockFile::load(root.path())
            .expect_err("appearance must be stale")
            .kind,
        LockErrorKind::Stale
    );

    build_lock(root.path()).expect("rebuild with local source");
    fs::remove_file(root.path().join("thp.local.toml")).expect("remove local source");
    assert_eq!(
        LockFile::load(root.path())
            .expect_err("disappearance must be stale")
            .kind,
        LockErrorKind::Stale
    );
}

#[test]
fn parser_borrows_extension_payload_without_decoding_toml() {
    let payload = "[this is deliberately not valid TOML";
    let bytes = minimal_lock(&format!(
        "extension example {}\n{}\nend-profile\nend-lock\n",
        payload.len(),
        payload
    ));
    let parsed = parse_lock(&bytes).expect("opaque payload must not be parsed");
    let raw = parsed.common.extensions[0].raw_toml;
    assert_eq!(raw, payload);
    assert!(
        (bytes.as_ptr() as usize..=bytes.as_ptr() as usize + bytes.len())
            .contains(&(raw.as_ptr() as usize))
    );
}

#[test]
fn extension_data_is_sorted_canonical_and_decoded_lazily() {
    let root = project(
        r#"
[extensions.zed]
value = 2

[extensions.alpha]
value = 1

[targets.cli.extensions.alpha]
value = 3

[targets.zed.memory]
limit = "3M"

[targets.alpha.memory]
limit = "4M"
"#,
    );
    build_lock(root.path()).expect("build lock");
    let text = fs::read_to_string(root.path().join("thp.lock")).expect("read lock");
    assert!(text.find("extension alpha").unwrap() < text.find("extension zed").unwrap());
    assert!(text.find("profile target alpha").unwrap() < text.find("profile target zed").unwrap());

    let lock = LockFile::load(root.path()).expect("load");
    let value: toml::Value = lock
        .select(Some("cli"))
        .extension("alpha")
        .expect("extension")
        .deserialize()
        .expect("decode");
    assert_eq!(value["value"].as_integer(), Some(3));
}

#[test]
fn rejects_unsupported_version_invalid_utf8_and_trailing_data() {
    let unsupported = minimal_lock("end-profile\nend-lock\n")
        .into_iter()
        .collect::<Vec<_>>();
    let mut unsupported = String::from_utf8(unsupported).expect("UTF-8");
    unsupported.replace_range(9..10, "9");
    assert_eq!(
        parse_lock(unsupported.as_bytes())
            .expect_err("unsupported")
            .kind,
        LockErrorKind::UnsupportedVersion
    );

    let mut invalid_utf8 = minimal_lock("end-profile\nend-lock\n");
    invalid_utf8.push(0xff);
    assert_eq!(
        parse_lock(&invalid_utf8).expect_err("invalid UTF-8").kind,
        LockErrorKind::InvalidUtf8
    );

    let trailing = minimal_lock("end-profile\nend-lock\ntrailing\n");
    assert_eq!(
        parse_lock(&trailing).expect_err("trailing data").kind,
        LockErrorKind::Corrupt
    );
}

#[test]
fn rejects_unknown_duplicate_missing_and_malformed_records() {
    let fixtures = [
        minimal_lock("unknown value\nend-profile\nend-lock\n"),
        minimal_lock("memory.limit 1\nend-profile\nend-lock\n"),
        minimal_lock("end-profile\nprofile common\nend-lock\n"),
        minimal_lock("extension x 999\nshort\n"),
        minimal_lock("extension x 0\n\nextension x 0\n\nend-profile\nend-lock\n"),
        minimal_lock("end-profile\nprofile target cli\nmemory.limit 1\n"),
        minimal_lock(
            "end-profile\n\
             profile target cli\n\
             memory.limit 1\n\
             request.post_max_size 1\n\
             request.max_stack_depth 1\n\
             request.max_open_handles 1\n\
             time.max_input 1\n\
             time.max_execution 1\n\
             end-profile\n\
             profile target cli\n",
        ),
    ];
    for fixture in fixtures {
        assert_eq!(
            parse_lock(&fixture).expect_err("corrupt fixture").kind,
            LockErrorKind::Corrupt
        );
    }
}

#[test]
fn missing_lock_is_reported_without_regeneration() {
    let root = project("");
    let error = LockFile::load(root.path()).expect_err("missing lock");
    assert_eq!(error.kind, LockErrorKind::Missing);
    assert!(!root.path().join("thp.lock").exists());
}

#[test]
fn atomic_replacement_leaves_complete_lock_and_no_temporary_file() {
    let root = project("[memory]\nlimit = \"1M\"\n");
    build_lock(root.path()).expect("first build");
    fs::write(root.path().join("thp.toml"), "[memory]\nlimit = \"2M\"\n").expect("change source");
    assert!(build_lock(root.path()).expect("replacement").changed);
    let lock = LockFile::load(root.path()).expect("replacement is complete");
    assert_eq!(
        lock.select(None).runtime.memory_limit,
        Limit::Finite(ByteSize::from_bytes(2 * 1024 * 1024))
    );
    let names = fs::read_dir(root.path())
        .expect("list project")
        .map(|entry| entry.expect("directory entry").file_name())
        .collect::<Vec<_>>();
    assert!(
        names
            .iter()
            .all(|name| !name.to_string_lossy().starts_with(".thp.lock."))
    );
}

#[cfg(unix)]
#[test]
fn lock_has_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let root = project("");
    build_lock(root.path()).expect("build lock");
    let mode = fs::metadata(root.path().join("thp.lock"))
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
}
