use std::{process::{Command, Stdio}, thread, time::Duration};

#[test]
fn mcp_stdio_process_starts_and_stays_running() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--transport", "stdio"])
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
    let mut child = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args([
            "mcp",
            "serve",
            "--transport",
            "http",
            "--host",
            "127.0.0.1",
            "--port",
            "8765",
        ])
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
