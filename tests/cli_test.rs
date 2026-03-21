use assert_cmd::Command;

fn lauyer() -> Command {
    Command::cargo_bin("lauyer").expect("binary should exist")
}

#[test]
fn help_exits_zero() {
    lauyer().arg("--help").assert().success();
}

#[test]
fn dgsi_courts_exits_zero_and_contains_stj() {
    let assert = lauyer().args(["dgsi", "courts"]).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("stj"), "output should list stj court");
}

#[test]
fn dgsi_search_help_exits_zero() {
    lauyer().args(["dgsi", "search", "--help"]).assert().success();
}

#[test]
fn unknown_subcommand_exits_nonzero() {
    lauyer().arg("nonexistent").assert().failure();
}

#[test]
fn format_flag_accepts_json() {
    // Just testing that the flag is accepted, not the output
    lauyer().args(["--format", "json", "dgsi", "courts"]).assert().success();
}

#[test]
fn quiet_flag_accepted() {
    lauyer().args(["--quiet", "dgsi", "courts"]).assert().success();
}
