use std::process::Command;

#[test]
fn help_lists_core_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .arg("--help")
        .output()
        .expect("run ctx --help");

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("setup"));
    assert!(text.contains("uninstall"));
    assert!(text.contains("doctor"));
    assert!(text.contains("update"));
    assert!(text.contains("config"));
    assert!(text.contains("memory"));
    assert!(text.contains("mcp"));
}

#[test]
fn mcp_help_lists_serve_and_transport_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--help"])
        .output()
        .expect("run ctx mcp serve --help");

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("--transport"));
    assert!(text.contains("stdio"));
    assert!(text.contains("http"));
}
