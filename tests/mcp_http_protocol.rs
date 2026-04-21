use std::{
    io::Read,
    net::TcpListener,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{Value, json};

const ACCEPT_BOTH: &str = "application/json, text/event-stream";

#[tokio::test]
async fn mcp_http_supports_core_mcp_flows() -> Result<()> {
    let port = unused_port()?;
    let base_url = format!("http://127.0.0.1:{port}/mcp");
    let mut child = spawn_http_server(port)?;
    let client = Client::new();

    let initialize = wait_for_initialize(&client, &base_url, &mut child).await?;
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

    let list_tools = post_session_request(
        &client,
        &base_url,
        &session_id,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }),
    )
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

    let config_show = post_session_request(
        &client,
        &base_url,
        &session_id,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "config_show",
                "arguments": {}
            }
        }),
    )
    .await?;
    assert_eq!(config_show.status(), 200);
    let config_show_message = parse_sse_json(&config_show.text().await?)?;
    assert_eq!(config_show_message["id"], 3);
    assert_eq!(config_show_message["result"]["isError"], false);
    assert_eq!(config_show_message["result"]["structuredContent"]["version"], 1);

    let list_resources = post_session_request(
        &client,
        &base_url,
        &session_id,
        &json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/list"
        }),
    )
    .await?;
    let list_resources_message = parse_sse_json(&list_resources.text().await?)?;
    let resource_uris = list_resources_message["result"]["resources"]
        .as_array()
        .context("resources/list missing resources array")?
        .iter()
        .filter_map(|resource| resource["uri"].as_str())
        .collect::<Vec<_>>();

    assert_eq!(list_resources_message["id"], 4);
    assert!(resource_uris.contains(&"ctx://config"));
    assert!(resource_uris.contains(&"ctx://paths"));
    assert!(resource_uris.contains(&"ctx://status"));

    let read_resource = post_session_request(
        &client,
        &base_url,
        &session_id,
        &json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {
                "uri": "ctx://config"
            }
        }),
    )
    .await?;
    let read_resource_message = parse_sse_json(&read_resource.text().await?)?;
    assert_eq!(read_resource_message["id"], 5);
    assert_eq!(
        read_resource_message["result"]["contents"][0]["uri"],
        "ctx://config"
    );
    assert!(
        read_resource_message["result"]["contents"][0]["text"]
            .as_str()
            .context("resources/read text content")?
            .contains("\"version\": 1")
    );

    let list_prompts = post_session_request(
        &client,
        &base_url,
        &session_id,
        &json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "prompts/list"
        }),
    )
    .await?;
    let list_prompts_message = parse_sse_json(&list_prompts.text().await?)?;
    let prompt_names = list_prompts_message["result"]["prompts"]
        .as_array()
        .context("prompts/list missing prompts array")?
        .iter()
        .filter_map(|prompt| prompt["name"].as_str())
        .collect::<Vec<_>>();

    assert_eq!(list_prompts_message["id"], 6);
    assert!(prompt_names.contains(&"memory-add-workflow"));
    assert!(prompt_names.contains(&"memory-search-workflow"));
    assert!(prompt_names.contains(&"setup-workflow"));

    let get_prompt = post_session_request(
        &client,
        &base_url,
        &session_id,
        &json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "prompts/get",
            "params": {
                "name": "setup-workflow",
                "arguments": {}
            }
        }),
    )
    .await?;
    let get_prompt_message = parse_sse_json(&get_prompt.text().await?)?;
    assert_eq!(get_prompt_message["id"], 7);
    assert_eq!(
        get_prompt_message["result"]["description"],
        "Guidance for initial ctx setup"
    );
    assert_eq!(get_prompt_message["result"]["messages"][0]["role"], "user");
    assert_eq!(
        get_prompt_message["result"]["messages"][1]["role"],
        "assistant"
    );

    shutdown(&mut child);
    Ok(())
}

#[tokio::test]
async fn mcp_http_rejects_non_initialize_post_without_session() -> Result<()> {
    let port = unused_port()?;
    let base_url = format!("http://127.0.0.1:{port}/mcp");
    let mut child = spawn_http_server(port)?;
    let client = Client::new();

    let _initialize = wait_for_initialize(&client, &base_url, &mut child).await?;

    let response = client
        .post(&base_url)
        .header(reqwest::header::ACCEPT, ACCEPT_BOTH)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "tools/list"
        }))
        .send()
        .await?;

    assert_eq!(response.status(), 400);
    assert_eq!(response.text().await?, "Bad Request: Session ID is required");

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

async fn wait_for_initialize(
    client: &Client,
    base_url: &str,
    child: &mut Child,
) -> Result<reqwest::Response> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().context("check HTTP MCP server status")? {
            return Err(anyhow::anyhow!(
                "HTTP MCP server exited before initialize: status={status}, stderr={}",
                read_stderr(child)
            ));
        }
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

async fn post_session_request(
    client: &Client,
    base_url: &str,
    session_id: &str,
    message: &Value,
) -> Result<reqwest::Response> {
    client
        .post(base_url)
        .header(reqwest::header::ACCEPT, ACCEPT_BOTH)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("Mcp-Session-Id", session_id)
        .json(message)
        .send()
        .await
        .context("send session request")
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

fn read_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(stream) = child.stderr.as_mut() {
        let _ = stream.read_to_string(&mut stderr);
    }
    stderr
}

fn shutdown(child: &mut Child) {
    child.kill().ok();
    let _ = child.wait();
}
