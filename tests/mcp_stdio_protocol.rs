use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::Value;

#[test]
fn mcp_stdio_supports_initialize_and_list_tools() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--transport", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn stdio mcp server");

    let mut stdin = child.stdin.take().expect("take child stdin");
    let stdout = child.stdout.take().expect("take child stdout");
    let mut reader = BufReader::new(stdout);

    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "ctx-test-client",
                    "version": "0.0.1"
                }
            }
        }),
    );
    let initialize = read_message(&mut reader);
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["serverInfo"]["name"], "ctx");

    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    );
    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }),
    );

    let list_tools = read_response(&mut reader, 2);
    let names = list_tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"memory_add"));
    assert!(names.contains(&"memory_search"));
    assert!(names.contains(&"setup_run"));

    shutdown(child);
}

fn send_message(stdin: &mut ChildStdin, message: &Value) {
    serde_json::to_writer(&mut *stdin, message).expect("serialize json-rpc message");
    writeln!(stdin).expect("terminate json-rpc message");
    stdin.flush().expect("flush payload");
}

fn read_response(reader: &mut BufReader<ChildStdout>, id: i64) -> Value {
    loop {
        let message = read_message(reader);
        if message["id"].as_i64() == Some(id) {
            return message;
        }
    }
}

fn read_message(reader: &mut BufReader<ChildStdout>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("read message line");
    serde_json::from_str(line.trim_end()).expect("parse json-rpc payload")
}

fn shutdown(mut child: Child) {
    child.kill().ok();
    let _ = child.wait();
}
