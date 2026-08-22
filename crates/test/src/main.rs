use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use thp_test::{Runner, RunnerOptions};

fn main() -> ExitCode {
    match arguments(env::args_os().skip(1)) {
        Ok(Arguments::Help) => {
            print_help();
            ExitCode::SUCCESS
        }
        Ok(Arguments::Run { options, paths }) => match Runner::new(options).run_paths(paths) {
            Ok(summary) => {
                for result in &summary.results {
                    let name = if result.name.is_empty() {
                        String::new()
                    } else {
                        format!(" - {}", result.name)
                    };
                    println!("{} {}{name}", result.status.label(), result.path.display());
                    if let Some(details) = &result.details {
                        for line in details.lines() {
                            println!("  {line}");
                        }
                    }
                }
                println!(
                    "Tests: {}, Pass: {}, Fail: {}, Skip: {}, Bork: {}",
                    summary.tests, summary.passed, summary.failed, summary.skipped, summary.borked
                );
                if summary.is_success() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(error) => {
                eprintln!("thp-test: {error}");
                ExitCode::from(2)
            }
        },
        Err(error) => {
            eprintln!("thp-test: {error}");
            eprintln!("Use `thp-test --help` for usage.");
            ExitCode::from(2)
        }
    }
}

enum Arguments {
    Help,
    Run {
        options: RunnerOptions,
        paths: Vec<PathBuf>,
    },
}

fn arguments(arguments: impl IntoIterator<Item = std::ffi::OsString>) -> Result<Arguments, String> {
    let mut options = RunnerOptions::default();
    let mut paths = Vec::new();
    for argument in arguments {
        let argument = argument
            .into_string()
            .map_err(|_| "arguments must be valid UTF-8".to_owned())?;
        if argument == "--help" {
            return Ok(Arguments::Help);
        }
        if let Some(value) = argument.strip_prefix("--max-instructions=") {
            if options.max_instructions.is_some() {
                return Err("`--max-instructions` may be supplied only once".to_owned());
            }
            options.max_instructions = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| "`--max-instructions` must be an integer".to_owned())?,
            );
        } else if argument.starts_with('-') {
            return Err(format!("unknown option `{argument}`"));
        } else {
            paths.push(PathBuf::from(argument));
        }
    }
    if paths.is_empty() {
        return Err("at least one file or directory path is required".to_owned());
    }
    Ok(Arguments::Run { options, paths })
}

fn print_help() {
    println!(
        "Usage: thp-test [--max-instructions=N] PATH...\n\
         \n\
         Recursively discovers .phpt files and executes them through the THP VM."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_options_and_paths() {
        let parsed =
            arguments(["--max-instructions=10".into(), "tests".into()]).expect("valid arguments");
        let Arguments::Run { options, paths } = parsed else {
            panic!("run arguments");
        };
        assert_eq!(options.max_instructions, Some(10));
        assert_eq!(paths, [PathBuf::from("tests")]);
    }

    #[test]
    fn rejects_invalid_invocations() {
        assert!(arguments(Vec::<std::ffi::OsString>::new()).is_err());
        assert!(arguments(["--unknown".into()]).is_err());
        assert!(arguments(["--max-instructions=nope".into(), "a".into()]).is_err());
    }
}
