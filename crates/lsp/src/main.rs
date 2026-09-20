use std::env;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.iter().any(|argument| argument == "--version") {
        println!("thp-lsp {VERSION}");
        return ExitCode::SUCCESS;
    }
    if arguments.iter().any(|argument| argument == "--help") {
        println!(
            "THP language server\n\nUsage: thp-lsp [--stdio | --help | --version]\n\nThe default transport is Language Server Protocol over stdio."
        );
        return ExitCode::SUCCESS;
    }
    if !(arguments.is_empty() || arguments.len() == 1 && arguments[0] == "--stdio") {
        eprintln!("unexpected argument; use `thp-lsp --help`");
        return ExitCode::from(1);
    }

    let (connection, threads) = lsp_server::Connection::stdio();
    let result = thp_lsp::run(&connection);
    drop(connection);
    let result = result.and_then(|()| {
        threads
            .join()
            .map_err(|error| format!("stdio transport failed: {error}"))
    });
    if let Err(error) = result {
        eprintln!("{error}");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
