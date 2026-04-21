use std::{
    net::TcpListener,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use tempfile::TempDir;

#[test]
fn mcp_stdio_process_starts_and_stays_running() {
    let data_dir = TempDir::new().expect("create temp data dir");
    let cache_dir = TempDir::new().expect("create temp cache dir");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--transport", "stdio"])
        .env("CTX_DATA_DIR", data_dir.path())
        .env("CTX_CACHE_DIR", cache_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn stdio mcp server");

    thread::sleep(Duration::from_millis(250));
    assert!(child.try_wait().expect("check child state").is_none());

    child.kill().ok();
    let _ = child.wait();
}

#[test]
fn mcp_http_process_starts_and_stays_running() {
    let data_dir = TempDir::new().expect("create temp data dir");
    let cache_dir = TempDir::new().expect("create temp cache dir");
    let port = unused_port();
    let mut child = Command::new(env!("CARGO_BIN_EXE_ctx"))
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
        .env("CTX_DATA_DIR", data_dir.path())
        .env("CTX_CACHE_DIR", cache_dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn http mcp server");

    thread::sleep(Duration::from_millis(250));
    assert!(child.try_wait().expect("check child state").is_none());

    child.kill().ok();
    let _ = child.wait();
}

fn unused_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("read local addr").port();
    drop(listener);
    port
}
