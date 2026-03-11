use std::process::Command;

fn bin() -> String {
    env!("CARGO_BIN_EXE_ply").to_string()
}

#[test]
fn validate_command_runs() {
    let output = Command::new(bin())
        .args(["validate", "tests/fixtures/sample_games.pgn"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("validated games: 2"));
}

#[test]
fn stats_json_command_runs() {
    let output = Command::new(bin())
        .args(["stats", "tests/fixtures/sample_games.pgn", "--json"])
        .output()
        .expect("should run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"games\": 2"));
}
