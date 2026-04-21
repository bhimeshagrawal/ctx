use std::{
    net::TcpListener,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{Value, json};

const ACCEPT_BOTH: &str = "application/json, text/event-stream";

#[tokio::test]
async fn mcp_http_supports_initialize_and_list_tools() -> Result<()> {
    let port = unused_port()?;
    let base_url = format!("http://127.0.0.1:{port}/mcp");
    let mut child = spawn_http_server(port)?;
    let client = Client::new();

    let initialize = wait_for_initialize(&client, &base_url).await?;
    let session_id = initialize
        .headers()
        .get("Mcp-Session-Id")
        .context("missing MCP session id header")?
        .to_str()
        .context("invalid MCP session id header")?
        .to_string();
    assert_eq!(initialize.status(), 200);
    assert!(
        header_contains(initialize.headers(), reqwest::header::CONTENT_TYPE, "text/event-stream"),
        "initialize should respond with SSE"
    );

    let initialize_body = initialize.text().await?;
    let initialize_message = parse_sse_json(&initialize_body)?;
    assert_eq!(initialize_message["id"], 1);
    assert_eq!(initialize_message["result"]["serverInfo"]["name"], "ctx");

    let initialized = client
        .post(&base_url)
        .header(reqwest::header::ACCEPT, ACCEPT_BOTH)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("Mcp-Session-Id", &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .send()
        .await?;
    assert_eq!(initialized.status(), 202);

    let list_tools = client
        .post(&base_url)
        .header(reqwest::header::ACCEPT, ACCEPT_BOTH)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("Mcp-Session-Id", &session_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }))
        .send()
        .await?;
    assert_eq!(list_tools.status(), 200);
    assert!(
        header_contains(list_tools.headers(), reqwest::header::CONTENT_TYPE, "text/event-stream"),
        "tools/list should respond with SSE"
    );

    let list_tools_body = list_tools.text().await?;
    let list_tools_message = parse_sse_json(&list_tools_body)?;
    let names = list_tools_message["result"]["tools"]
        .as_array()
        .context("tools/list missing tools array")?
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();

    assert_eq!(list_tools_message["id"], 2);
    assert!(names.contains(&"memory_add"));
    assert!(names.contains(&"memory_search"));
    assert!(names.contains(&"setup_run"));

    shutdown(&mut child);
    Ok(())
}

fn spawn_http_server(port: u16) -> Result<Child> {
    Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args([
            "mcp",
            "serve",
            "--transport",
            "http",
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn http mcp server")
}

async fn wait_for_initialize(client: &Client, base_url: &str) -> Result<reqwest::Response> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match client
            .post(base_url)
            .header(reqwest::header::ACCEPT, ACCEPT_BOTH)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&json!({
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
            }))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => return Ok(response),
            Ok(_) | Err(_) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok(response) => return Err(anyhow::anyhow!("initialize failed with status {}", response.status())),
            Err(error) => return Err(error).context("initialize request did not reach the HTTP MCP server"),
        }
    }
}

fn parse_sse_json(body: &str) -> Result<Value> {
    for payload in body.lines().filter_map(|line| line.strip_prefix("data: ")) {
        let payload = payload.trim();
        if payload.is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str(payload) {
            return Ok(json);
        }
    }
    Err(anyhow::anyhow!("parse SSE JSON payload")).context(format!("response body was: {body}"))
}

fn header_contains(headers: &reqwest::header::HeaderMap, name: reqwest::header::HeaderName, needle: &str) -> bool {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains(needle))
}

fn unused_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").context("bind ephemeral port")?;
    let port = listener
        .local_addr()
        .context("read ephemeral port")?
        .port();
    drop(listener);
    Ok(port)
}

fn shutdown(child: &mut Child) {
    child.kill().ok();
    let _ = child.wait();
}
