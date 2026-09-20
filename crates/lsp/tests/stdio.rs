use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

fn send(input: &mut impl Write, value: &Value) {
    let body = value.to_string();
    write!(input, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
    input.flush().unwrap();
}

fn receive(output: &mut impl BufRead) -> Value {
    let mut length = None;
    loop {
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        assert!(!line.is_empty(), "language server closed stdout");
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            length = Some(value.trim().parse::<usize>().unwrap());
        }
    }
    let mut body = vec![0; length.unwrap()];
    output.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn response(output: &mut impl BufRead, id: i64) -> Value {
    loop {
        let message = receive(output);
        if message["id"] == id {
            return message;
        }
    }
}

#[test]
fn exit_without_shutdown_returns_failure() {
    for initialize in [false, true] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_thp-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        if initialize {
            send(
                &mut input,
                &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}),
            );
            assert!(response(&mut output, 1).get("error").is_none());
            send(
                &mut input,
                &json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
            );
        }
        send(
            &mut input,
            &json!({"jsonrpc":"2.0","method":"exit","params":null}),
        );
        drop(input);
        assert_eq!(
            child.wait().unwrap().code(),
            Some(1),
            "initialized: {initialize}"
        );
    }
}

#[test]
fn serves_every_advertised_request_and_recovers_from_bad_params() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_thp-lsp"))
        .arg("--stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());

    send(
        &mut input,
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}),
    );
    let initialized = response(&mut output, 1);
    assert_eq!(initialized["result"]["serverInfo"]["version"], "0.1.0");
    send(
        &mut input,
        &json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
    );
    send(
        &mut input,
        &json!({
            "jsonrpc":"2.0",
            "method":"textDocument/didOpen",
            "params":{"textDocument":{
                "uri":"file:///stdio.thp",
                "languageId":"thp",
                "version":1,
                "text":"<?thp\nfunction greet(string $name): string { return $name; }\necho greet(\"Ada\");\n"
            }}
        }),
    );

    let document = json!({"textDocument":{"uri":"file:///stdio.thp"}});
    let positioned =
        json!({"textDocument":{"uri":"file:///stdio.thp"},"position":{"line":1,"character":10}});
    let requests = [
        ("textDocument/hover", positioned.clone()),
        (
            "textDocument/completion",
            json!({"textDocument":{"uri":"file:///stdio.thp"},"position":{"line":2,"character":5},"context":{"triggerKind":1}}),
        ),
        (
            "textDocument/signatureHelp",
            json!({"textDocument":{"uri":"file:///stdio.thp"},"position":{"line":2,"character":14}}),
        ),
        ("textDocument/definition", positioned.clone()),
        (
            "textDocument/references",
            json!({"textDocument":{"uri":"file:///stdio.thp"},"position":{"line":1,"character":10},"context":{"includeDeclaration":true}}),
        ),
        ("textDocument/prepareRename", positioned.clone()),
        (
            "textDocument/rename",
            json!({"textDocument":{"uri":"file:///stdio.thp"},"position":{"line":1,"character":10},"newName":"welcome"}),
        ),
        ("textDocument/documentSymbol", document.clone()),
        ("workspace/symbol", json!({"query":"greet"})),
        ("textDocument/semanticTokens/full", document.clone()),
        (
            "textDocument/formatting",
            json!({"textDocument":{"uri":"file:///stdio.thp"},"options":{"tabSize":4,"insertSpaces":true}}),
        ),
    ];
    for (index, (method, params)) in requests.into_iter().enumerate() {
        let id = i64::try_from(index).unwrap() + 2;
        send(
            &mut input,
            &json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}),
        );
        assert!(response(&mut output, id).get("error").is_none(), "{method}");
    }

    send(
        &mut input,
        &json!({"jsonrpc":"2.0","id":20,"method":"textDocument/hover","params":{}}),
    );
    assert_eq!(response(&mut output, 20)["error"]["code"], -32602);
    send(
        &mut input,
        &json!({"jsonrpc":"2.0","id":21,"method":"textDocument/hover","params":positioned}),
    );
    assert!(response(&mut output, 21).get("error").is_none());
    send(
        &mut input,
        &json!({"jsonrpc":"2.0","id":22,"method":"shutdown","params":null}),
    );
    assert!(response(&mut output, 22).get("error").is_none());
    send(
        &mut input,
        &json!({"jsonrpc":"2.0","method":"exit","params":null}),
    );
    drop(input);
    assert!(child.wait().unwrap().success());
}
