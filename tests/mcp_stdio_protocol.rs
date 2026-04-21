use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::Value;
use tempfile::TempDir;

#[test]
fn mcp_stdio_supports_core_mcp_flows() {
    let data_dir = TempDir::new().expect("create temp data dir");
    let cache_dir = TempDir::new().expect("create temp cache dir");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--transport", "stdio"])
        .env("CTX_DATA_DIR", data_dir.path())
        .env("CTX_CACHE_DIR", cache_dir.path())
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

    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "config_show",
                "arguments": {}
            }
        }),
    );
    let config_show = read_response(&mut reader, 3);
    assert_eq!(config_show["result"]["isError"], false);
    assert_eq!(config_show["result"]["structuredContent"]["version"], 1);
    assert_eq!(
        config_show["result"]["content"][0]["type"],
        "text"
    );

    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/list"
        }),
    );
    let list_resources = read_response(&mut reader, 4);
    let resource_uris = list_resources["result"]["resources"]
        .as_array()
        .expect("resources array")
        .iter()
        .filter_map(|resource| resource["uri"].as_str())
        .collect::<Vec<_>>();

    assert!(resource_uris.contains(&"ctx://config"));
    assert!(resource_uris.contains(&"ctx://paths"));
    assert!(resource_uris.contains(&"ctx://status"));

    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {
                "uri": "ctx://config"
            }
        }),
    );
    let read_resource = read_response(&mut reader, 5);
    assert_eq!(
        read_resource["result"]["contents"][0]["uri"],
        "ctx://config"
    );
    assert_eq!(
        read_resource["result"]["contents"][0]["mimeType"],
        "application/json"
    );
    assert!(
        read_resource["result"]["contents"][0]["text"]
            .as_str()
            .expect("resource text")
            .contains("\"version\": 1")
    );

    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "prompts/list"
        }),
    );
    let list_prompts = read_response(&mut reader, 6);
    let prompt_names = list_prompts["result"]["prompts"]
        .as_array()
        .expect("prompts array")
        .iter()
        .filter_map(|prompt| prompt["name"].as_str())
        .collect::<Vec<_>>();

    assert!(prompt_names.contains(&"memory-add-workflow"));
    assert!(prompt_names.contains(&"memory-search-workflow"));
    assert!(prompt_names.contains(&"setup-workflow"));

    send_message(
        &mut stdin,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "prompts/get",
            "params": {
                "name": "setup-workflow",
                "arguments": {}
            }
        }),
    );
    let get_prompt = read_response(&mut reader, 7);
    assert_eq!(
        get_prompt["result"]["description"],
        "Guidance for initial ctx setup"
    );
    assert_eq!(get_prompt["result"]["messages"][0]["role"], "user");
    assert_eq!(get_prompt["result"]["messages"][1]["role"], "assistant");

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
