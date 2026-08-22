#![allow(clippy::too_many_lines)]

use std::alloc::System;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use thp_compiler::{
    Compilation, ProjectCompilation, ProjectRequest, cache_warm_project, compile_path,
    compile_path_cached, compile_project, load_frozen_project,
};
use thp_metrics::{Metrics, Stage, TrackingAllocator};
use thp_opcache::Store;
use thp_vm::{ExecutionContext, Limits};

#[global_allocator]
static ALLOCATOR: TrackingAllocator<System> = TrackingAllocator::new(System);

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run(arguments: Vec<std::ffi::OsString>) -> Result<(), String> {
    let arguments = arguments
        .into_iter()
        .map(|argument| {
            argument
                .into_string()
                .map_err(|_| "arguments must be valid UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if arguments.is_empty() || arguments.iter().any(|argument| argument == "--help") {
        print_help();
        return Ok(());
    }
    if arguments.iter().any(|argument| argument == "--version") {
        println!("thp {VERSION}");
        return Ok(());
    }

    let metrics_format = option_value(&arguments, "--metrics").unwrap_or("off");
    if !matches!(metrics_format, "off" | "human" | "json") {
        return Err("`--metrics` must be `off`, `human`, or `json`".to_owned());
    }
    let positional = arguments
        .iter()
        .filter(|argument| !argument.starts_with("--"))
        .collect::<Vec<_>>();
    let Some(command) = positional.first().copied() else {
        return Err("missing command; use `thp --help`".to_owned());
    };
    let path = positional.get(1).map(|value| PathBuf::from(*value));
    let project_root = option_value(&arguments, "--project")
        .map(PathBuf::from)
        .map_or_else(env::current_dir, Ok)
        .map_err(|error| format!("cannot determine project root: {error}"))?;
    match command.as_str() {
        "check" => {
            let path = path.ok_or_else(|| "`thp check` requires a source file".to_owned())?;
            let compilation = load_selected_compilation(path, &project_root)?;
            require_selected_success(&compilation)?;
            print_metrics(compilation.metrics(), metrics_format)?;
            Ok(())
        }
        "inspect" => {
            let path = path.ok_or_else(|| "`thp inspect` requires a source file".to_owned())?;
            let emit = option_value(&arguments, "--emit").unwrap_or("ast");
            let compilation = load_selected_compilation(path, &project_root)?;
            require_selected_frontend(&compilation)?;
            inspect(&compilation, emit)?;
            print_metrics(compilation.metrics(), metrics_format)?;
            Ok(())
        }
        "run" => {
            let path = path.ok_or_else(|| "`thp run` requires a source file".to_owned())?;
            let engine = option_value(&arguments, "--engine").unwrap_or("auto");
            if !matches!(engine, "auto" | "vm" | "jit") {
                return Err("`--engine` must be `auto`, `vm`, or `jit`".to_owned());
            }
            let max_instructions = option_value(&arguments, "--max-instructions")
                .map(|value| {
                    value
                        .parse::<u64>()
                        .map_err(|_| "`--max-instructions` must be an integer".to_owned())
                })
                .transpose()?;
            let opcache = option_value(&arguments, "--opcache").unwrap_or("off");
            let frozen = arguments.iter().any(|argument| argument == "--frozen");
            let project_mode = project_root.join("thp.toml").is_file();
            let (source, bytecode, mut metrics) = if frozen {
                if !project_mode {
                    return Err("`run --frozen` requires a project `thp.toml`".to_owned());
                }
                if opcache == "off" {
                    return Err("`run --frozen` requires `--opcache=PATH`".to_owned());
                }
                let prepared = load_frozen_project(
                    &ProjectRequest::new(&project_root, &path),
                    &Store::new(opcache),
                )
                .map_err(|error| error.to_string())?;
                let source = prepared
                    .sources
                    .get(prepared.entry_source)
                    .expect("prepared entry source exists")
                    .clone();
                (source, prepared.bytecode, Metrics::default())
            } else if project_mode {
                let request = ProjectRequest::new(&project_root, &path);
                let compilation = if opcache == "off" {
                    compile_project(&request).map_err(|error| error.to_string())?
                } else {
                    cache_warm_project(&request, &Store::new(opcache))
                        .map_err(|error| error.to_string())?
                        .0
                };
                if !compilation.is_success() {
                    return Err(compilation.rendered_diagnostics());
                }
                let entry = compilation
                    .units
                    .iter()
                    .find(|unit| unit.module.is_entry)
                    .expect("project compilation has an entry");
                (
                    entry.source.clone(),
                    compilation
                        .bytecode
                        .expect("successful compilation has bytecode"),
                    compilation.metrics,
                )
            } else if opcache == "off" {
                let compilation = load_compilation(path)?;
                require_success(&compilation)?;
                (
                    compilation.source,
                    compilation
                        .bytecode
                        .expect("successful compilation has bytecode"),
                    compilation.metrics,
                )
            } else {
                let cached = compile_path_cached(path, &Store::new(opcache), b"")
                    .map_err(|error| error.to_string())?;
                if !cached.is_success() {
                    return Err(cached.rendered_diagnostics());
                }
                (
                    cached.source,
                    cached
                        .bytecode
                        .expect("successful cached compilation has bytecode"),
                    cached.metrics,
                )
            };
            let use_jit = engine == "jit"
                || (engine == "auto" && max_instructions.is_none() && thp_jit::supports(&bytecode));
            let stdout = io::stdout();
            let mut output = stdout.lock();
            if use_jit {
                if max_instructions.is_some() {
                    return Err(
                        "`--max-instructions` requires `--engine=vm` or auto fallback".to_owned(),
                    );
                }
                let execution = metrics
                    .measure(Stage::Jit, || thp_jit::execute_to(&bytecode, &mut output))
                    .map_err(|error| error.to_string())?;
                if let Some(measurement) = metrics.last_mut() {
                    measurement.set_output(
                        execution.compiled_functions,
                        usize::try_from(execution.output_bytes).unwrap_or(usize::MAX),
                    );
                }
            } else {
                let context = ExecutionContext {
                    limits: Limits {
                        max_instructions,
                        max_execution: None,
                        ..Limits::default()
                    },
                    ..ExecutionContext::default()
                };
                #[allow(
                    clippy::result_large_err,
                    reason = "the VM failure preserves request statistics for diagnostics"
                )]
                let execution = metrics.measure(Stage::Vm, || {
                    thp_vm::execute_to(&bytecode, &context, &mut output)
                });
                let execution = execution.map_err(|failure| match failure.error {
                    thp_vm::VmError::Runtime(runtime) => {
                        let (line, column) = source.line_column(runtime.span.start as usize);
                        format!("{}:{line}:{column}: {runtime}", source.path().display())
                    }
                    other => other.to_string(),
                })?;
                if let Some(measurement) = metrics.last_mut() {
                    measurement.set_output(
                        usize::try_from(execution.instructions).unwrap_or(usize::MAX),
                        usize::try_from(execution.output_bytes).unwrap_or(usize::MAX),
                    );
                }
            }
            output
                .flush()
                .map_err(|error| format!("failed to flush program output: {error}"))?;
            drop(output);
            print_metrics(&metrics, metrics_format)?;
            Ok(())
        }
        "cache-warm" => {
            let path = path.ok_or_else(|| "`thp cache-warm` requires a source file".to_owned())?;
            if !project_root.join("thp.toml").is_file() {
                return Err("`thp cache-warm` requires a project `thp.toml`".to_owned());
            }
            let cache = option_value(&arguments, "--opcache")
                .ok_or_else(|| "`thp cache-warm` requires `--opcache=PATH`".to_owned())?;
            if cache == "off" {
                return Err("cannot warm a disabled OPcache".to_owned());
            }
            let (compilation, manifest) = cache_warm_project(
                &ProjectRequest::new(&project_root, path),
                &Store::new(cache),
            )
            .map_err(|error| error.to_string())?;
            if !compilation.is_success() {
                return Err(compilation.rendered_diagnostics());
            }
            let manifest = manifest.expect("successful warm-up publishes a manifest");
            println!(
                "warmed {} modules and published entry `{}`",
                compilation.units.len(),
                manifest.entry_id
            );
            print_metrics(&compilation.metrics, metrics_format)
        }
        "cache-prune" => {
            let cache = option_value(&arguments, "--opcache")
                .ok_or_else(|| "`thp cache-prune` requires `--opcache=PATH`".to_owned())?;
            if cache == "off" {
                return Err("cannot prune a disabled OPcache".to_owned());
            }
            let maximum_bytes = option_value(&arguments, "--max-bytes")
                .unwrap_or("268435456")
                .parse::<u64>()
                .map_err(|_| "`--max-bytes` must be an integer".to_owned())?;
            let mut metrics = Metrics::default();
            let result = metrics
                .measure(Stage::Cache, || Store::new(cache).prune(maximum_bytes))
                .map_err(|error| format!("cannot prune OPcache: {error}"))?;
            if let Some(measurement) = metrics.last_mut() {
                measurement.set_output(
                    result.removed_entries,
                    usize::try_from(result.removed_bytes).unwrap_or(usize::MAX),
                );
            }
            println!(
                "removed {} entries ({} bytes); {} bytes remain",
                result.removed_entries, result.removed_bytes, result.remaining_bytes
            );
            print_metrics(&metrics, metrics_format)
        }
        _ => Err(format!("unknown command `{command}`; use `thp --help`")),
    }
}

fn load_compilation(path: PathBuf) -> Result<Compilation, String> {
    compile_path(path).map_err(|error| error.to_string())
}

enum SelectedCompilation {
    Single(Compilation),
    Project(ProjectCompilation),
}

impl SelectedCompilation {
    fn metrics(&self) -> &Metrics {
        match self {
            Self::Single(compilation) => &compilation.metrics,
            Self::Project(compilation) => &compilation.metrics,
        }
    }
}

fn load_selected_compilation(
    path: PathBuf,
    project_root: &std::path::Path,
) -> Result<SelectedCompilation, String> {
    if project_root.join("thp.toml").is_file() {
        compile_project(&ProjectRequest::new(project_root, path))
            .map(SelectedCompilation::Project)
            .map_err(|error| error.to_string())
    } else {
        load_compilation(path).map(SelectedCompilation::Single)
    }
}

fn require_selected_success(compilation: &SelectedCompilation) -> Result<(), String> {
    match compilation {
        SelectedCompilation::Single(compilation) => require_success(compilation),
        SelectedCompilation::Project(compilation) if compilation.is_success() => Ok(()),
        SelectedCompilation::Project(compilation) => Err(compilation.rendered_diagnostics()),
    }
}

fn require_selected_frontend(compilation: &SelectedCompilation) -> Result<(), String> {
    match compilation {
        SelectedCompilation::Single(compilation) if compilation.diagnostics.is_empty() => Ok(()),
        SelectedCompilation::Single(compilation) => Err(compilation.rendered_diagnostics()),
        SelectedCompilation::Project(compilation) if compilation.diagnostics.is_empty() => Ok(()),
        SelectedCompilation::Project(compilation) => Err(compilation.rendered_diagnostics()),
    }
}

fn inspect(compilation: &SelectedCompilation, emit: &str) -> Result<(), String> {
    match (compilation, emit) {
        (SelectedCompilation::Single(value), "tokens") => println!("{:#?}", value.tokens),
        (SelectedCompilation::Single(value), "ast") => println!("{:#?}", value.ast),
        (SelectedCompilation::Project(value), "tokens") => {
            for unit in &value.units {
                println!("module {}:\n{:#?}", unit.module.id, unit.tokens);
            }
        }
        (SelectedCompilation::Project(value), "ast") => {
            for unit in &value.units {
                println!("module {}:\n{:#?}", unit.module.id, unit.ast);
            }
        }
        (SelectedCompilation::Project(value), "module-graph") => print!(
            "{}",
            value
                .graph
                .as_ref()
                .ok_or_else(|| "module graph was not produced".to_owned())?
        ),
        (SelectedCompilation::Project(value), "interfaces") => {
            for interface in &value.interfaces {
                println!(
                    "module {} namespace {} interface {}",
                    interface.module,
                    if interface.namespace.is_empty() {
                        "<global>"
                    } else {
                        &interface.namespace
                    },
                    interface.interface_hash
                );
                for export in &interface.exports {
                    println!("  {} {}", export.kind.as_str(), export.name);
                }
            }
        }
        (SelectedCompilation::Single(_), "module-graph" | "interfaces") => {
            return Err(
                "`module-graph` and `interfaces` inspection requires a project `thp.toml`"
                    .to_owned(),
            );
        }
        (SelectedCompilation::Single(value), "hir") => println!(
            "{:#?}",
            value
                .hir
                .as_ref()
                .ok_or_else(|| "HIR was not produced".to_owned())?
        ),
        (SelectedCompilation::Project(value), "hir") => println!(
            "{:#?}",
            value
                .hir
                .as_ref()
                .ok_or_else(|| "HIR was not produced".to_owned())?
        ),
        (SelectedCompilation::Single(value), "mir") => print!(
            "{}",
            value
                .mir
                .as_ref()
                .ok_or_else(|| "MIR was not produced".to_owned())?
        ),
        (SelectedCompilation::Project(value), "mir") => print!(
            "{}",
            value
                .mir
                .as_ref()
                .ok_or_else(|| "MIR was not produced".to_owned())?
        ),
        (SelectedCompilation::Single(value), "bytecode") => print!(
            "{}",
            value
                .bytecode
                .as_ref()
                .ok_or_else(|| "bytecode was not produced".to_owned())?
        ),
        (SelectedCompilation::Project(value), "bytecode") => print!(
            "{}",
            value
                .bytecode
                .as_ref()
                .ok_or_else(|| "bytecode was not produced".to_owned())?
        ),
        (_, _) => {
            return Err(
                "`--emit` must be `tokens`, `ast`, `interfaces`, `module-graph`, `hir`, `mir`, or `bytecode`"
                    .to_owned(),
            );
        }
    }
    Ok(())
}

fn require_success(compilation: &Compilation) -> Result<(), String> {
    if compilation.is_success() {
        Ok(())
    } else {
        Err(compilation.rendered_diagnostics())
    }
}

fn option_value<'arguments>(
    arguments: &'arguments [String],
    name: &str,
) -> Option<&'arguments str> {
    arguments.iter().find_map(|argument| {
        argument
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('='))
    })
}

fn print_metrics(metrics: &Metrics, format: &str) -> Result<(), String> {
    let mut stderr = io::stderr().lock();
    match format {
        "off" => Ok(()),
        "human" => writeln!(stderr, "{metrics}")
            .map_err(|error| format!("failed to write metrics: {error}")),
        "json" => writeln!(stderr, "{}", metrics.to_json())
            .map_err(|error| format!("failed to write metrics: {error}")),
        _ => unreachable!("metrics format validated"),
    }
}

fn print_help() {
    println!(
        "\
THP standalone compiler and interpreter

Usage:
  thp check [--project=DIR] [--metrics=off|human|json] FILE
  thp inspect [--project=DIR] [--emit=tokens|ast|interfaces|module-graph|hir|mir|bytecode] [--metrics=...] FILE
  thp run [--project=DIR] [--engine=auto|vm|jit] [--opcache=off|PATH] [--max-instructions=N] [--metrics=...] FILE
  thp cache-warm --opcache=PATH [--project=DIR] FILE
  thp run --frozen --opcache=PATH [--project=DIR] FILE
  thp cache-prune --opcache=PATH [--max-bytes=N] [--metrics=...]
  thp --version
"
    );
}
