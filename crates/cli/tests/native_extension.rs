#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn compile_library(root: &Path, mode: &str) -> PathBuf {
    let header = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates directory")
        .join("abi/include");
    let source = root.join("demo.c");
    fs::write(
        &source,
        format!(
            r#"#include "thp.h"
#include <string.h>
static ThpHost host;
static int32_t add_one(void *data, ThpCall *call, const ThpValue *const *args, size_t count, const ThpValue **result) {{
    int64_t value;
    if (count != 1 || ({integer_check} && host.value_get_int(call, args[0], &value) != 0)) return -1;
    {callback}
    return 0;
}}
static int32_t initialize(const ThpHost *provided) {{
    host = *provided;
    static const uint8_t name[] = "{function_name}";
    static const uint8_t int_type[] = "int";
    static const uint8_t object_type[] = "Base";
    static const uint8_t bad_type[] = "vector<";
    ThpBorrowedBuffer parameters[] = {{{{ {parameter}, {parameter_length} }}}};
    ThpNativeFunction function = {{ sizeof(ThpNativeFunction),
        {{ name, sizeof(name) - 1 }}, parameters, 1,
        {{ {result_type}, {result_type_length} }}, add_one, 0 }};
    return host.register_function(host.user_data, &function);
}}
static const ThpExtension extension = {{ THP_ABI_VERSION, {descriptor_size}, "demo", initialize, 0 }};
const ThpExtension *thp_extension_entry(void) {{ return &extension; }}
"#,
            callback = match mode {
                "subtype-result" => "*result = host.value_clone(call, args[0]);",
                "expired-handle" => "static const ThpValue *saved; if (!saved) { saved = args[0]; *result = host.value_new_int(call, 1); } else { host.value_new_int(call, 2); *result = saved; }",
                "wrong-result" => "static const uint8_t text[] = \"wrong\"; *result = host.value_new_bytes(call, (ThpBorrowedBuffer){ text, sizeof(text) - 1 });",
                "invalid-handle" => "*result = (const ThpValue *)(uintptr_t)1;",
                "cloned-result" => "const ThpValue *temporary = host.value_new_int(call, value + 1); *result = host.value_clone(call, temporary);",
                _ => "*result = host.value_new_int(call, value + 1);",
            },
            integer_check = mode != "subtype-result",
            function_name = if mode == "subtype-result" { "identity_object" } else { "add_one" },
            parameter = match mode { "bad-signature" => "bad_type", "subtype-result" => "object_type", _ => "int_type" },
            parameter_length = match mode { "bad-signature" => "sizeof(bad_type) - 1", "subtype-result" => "sizeof(object_type) - 1", _ => "sizeof(int_type) - 1" },
            result_type = if mode == "subtype-result" { "object_type" } else { "int_type" },
            result_type_length = if mode == "subtype-result" { "sizeof(object_type) - 1" } else { "sizeof(int_type) - 1" },
            descriptor_size = if mode == "short-descriptor" { "sizeof(uint32_t)" } else { "sizeof(ThpExtension)" },
        ),
    )
    .expect("write C extension");
    let library = root.join("libdemo.so");
    let output = Command::new("cc")
        .args(["-shared", "-fPIC", "-I"])
        .arg(header)
        .arg(&source)
        .arg("-o")
        .arg(&library)
        .output()
        .expect("compile C extension");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    library
}

#[test]
fn native_function_accepts_subclass_result_for_base_return_type() {
    let root = tempfile::tempdir().expect("project");
    compile_library(root.path(), "subtype-result");
    fs::write(
        root.path().join("thp.toml"),
        "[extension_libraries]\ndemo = 'libdemo.so'\n",
    )
    .expect("write config");
    fs::write(
        root.path().join("main.thp"),
        "<?thp\nclass Base {}\nclass Child extends Base {}\nidentity_object(new Child());\necho 'ok';\n",
    )
    .expect("write source");

    let output = thp(root.path(), &["run", "--engine=vm", "main.thp"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ok");
}

#[test]
fn native_value_handles_expire_between_calls() {
    let root = tempfile::tempdir().expect("project");
    compile_library(root.path(), "expired-handle");
    fs::write(
        root.path().join("thp.toml"),
        "[extension_libraries]\ndemo = 'libdemo.so'\n",
    )
    .expect("write config");
    fs::write(
        root.path().join("main.thp"),
        "<?thp\necho add_one(41);\necho add_one(99);\n",
    )
    .expect("write source");

    let output = thp(root.path(), &["run", "--engine=vm", "main.thp"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid value handle"));
}

fn thp(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_thp"))
        .current_dir(root)
        .args(arguments)
        .output()
        .expect("run thp")
}

#[test]
fn native_calls_check_types_and_changed_frozen_libraries() {
    let root = tempfile::tempdir().expect("project");
    compile_library(root.path(), "valid");
    fs::write(
        root.path().join("thp.toml"),
        "[extension_libraries]\ndemo = 'libdemo.so'\n",
    )
    .expect("write config");
    fs::write(root.path().join("main.thp"), "<?thp\necho add_one(41);\n").expect("write source");

    let output = thp(root.path(), &["run", "--engine=vm", "main.thp"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"42");

    let cache = format!("--opcache={}", root.path().join("cache").display());
    let warm = thp(root.path(), &["cache-warm", &cache, "main.thp"]);
    assert!(
        warm.status.success(),
        "{}",
        String::from_utf8_lossy(&warm.stderr)
    );
    let frozen = thp(
        root.path(),
        &["run", "--frozen", "--engine=vm", &cache, "main.thp"],
    );
    assert!(
        frozen.status.success(),
        "{}",
        String::from_utf8_lossy(&frozen.stderr)
    );
    assert_eq!(frozen.stdout, b"42");

    compile_library(root.path(), "wrong-result");
    let changed = thp(
        root.path(),
        &["run", "--frozen", "--engine=vm", &cache, "main.thp"],
    );
    assert!(!changed.status.success());
    assert!(String::from_utf8_lossy(&changed.stderr).contains("frozen"));

    let wrong = thp(root.path(), &["run", "--engine=vm", "main.thp"]);
    assert!(!wrong.status.success());
    assert!(String::from_utf8_lossy(&wrong.stderr).contains("returned string, expected int"));

    compile_library(root.path(), "invalid-handle");
    let invalid = thp(root.path(), &["run", "--engine=vm", "main.thp"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid value handle"));

    compile_library(root.path(), "cloned-result");
    let cloned = thp(root.path(), &["run", "--engine=vm", "main.thp"]);
    assert!(
        cloned.status.success(),
        "{}",
        String::from_utf8_lossy(&cloned.stderr)
    );
    assert_eq!(cloned.stdout, b"42");

    compile_library(root.path(), "bad-signature");
    let bad = thp(root.path(), &["check", "main.thp"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("invalid native type"));

    compile_library(root.path(), "short-descriptor");
    let short = thp(root.path(), &["check", "main.thp"]);
    assert!(!short.status.success());
    assert!(String::from_utf8_lossy(&short.stderr).contains("incompatible extension descriptor"));
}
