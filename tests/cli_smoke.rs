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
}
