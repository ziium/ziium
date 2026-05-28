use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn write_temp_program(contents: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should work")
        .as_nanos();
    let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    let path = env::temp_dir().join(format!("ziium_cli_{process}_{unique}_{count}.zm"));
    fs::write(&path, contents).expect("temp program should be writable");
    path
}

#[test]
fn cli_runs_program_file() {
    let path = write_temp_program(
        r#"이름은 "철수"이다
이름을 출력한다"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .arg(&path)
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "철수\n");

    let _ = fs::remove_file(path);
}

#[test]
fn cli_prints_tokens() {
    let path = write_temp_program("이름은 \"철수\"이다");

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["tokens", path.to_str().expect("utf-8 path")])
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Ident(\"이름\")"));
    assert!(stdout.contains("Topic(\"은\")"));
    assert!(stdout.contains("Copula(\"이다\")"));

    let _ = fs::remove_file(path);
}

#[test]
fn cli_prints_hir() {
    let path = write_temp_program("문장은 \"지음\"으로 인사만들기이다.");

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["hir", path.to_str().expect("utf-8 path")])
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Send"));
    assert!(stdout.contains("selector: Transform("));
    assert!(stdout.contains("\"인사만들기\""));

    let _ = fs::remove_file(path);
}

#[test]
fn cli_reports_tagged_runtime_diagnostic() {
    let path = write_temp_program(
        r#"값은 1이다
값()
"끝"을 출력한다"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .arg(&path)
        .output()
        .expect("cli should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[실행 오류]"));
    assert!(stderr.contains("위치: 2번째 줄 2번째 열"));
    assert!(stderr.contains("메시지: 호출할 수 없는 값을 호출했습니다."));
    assert!(stderr.contains("코드:"));
    assert!(stderr.contains("1 | 값은 1이다"));
    assert!(stderr.contains("2 | 값()"));
    assert!(stderr.contains("^"));
    assert!(stderr.contains("3 | \"끝\"을 출력한다"));

    let _ = fs::remove_file(path);
}

#[test]
fn cli_repl_runs_persistent_session() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("repl should start");

    child
        .stdin
        .as_mut()
        .expect("stdin should be available")
        .write_all(
            r#"이름은 "철수"이다
이름을 출력한다
:quit
"#
            .as_bytes(),
        )
        .expect("repl input should be writable");

    let output = child.wait_with_output().expect("repl should finish");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "철수\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("ziium> "));
}

#[test]
fn cli_repl_runs_block_after_blank_line() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("repl should start");

    child
        .stdin
        .as_mut()
        .expect("stdin should be available")
        .write_all(
            r#"나이는 20이다
나이 >= 20이면
  "성인"을 출력한다

:quit
"#
            .as_bytes(),
        )
        .expect("repl input should be writable");

    let output = child.wait_with_output().expect("repl should finish");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "성인\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "안내: 다음 줄부터 두 칸 들여써 블록을 입력하세요. 빈 줄을 입력하면 실행합니다."
    ));
}

#[test]
fn cli_prints_ast() {
    let path = write_temp_program("이름은 \"철수\"이다");

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["ast", path.to_str().expect("utf-8 path")])
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Bind"));
    assert!(stdout.contains("이름"));

    let _ = fs::remove_file(path);
}

#[test]
fn cli_prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .arg("--version")
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ziium"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .arg("--help")
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("사용법"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("repl"));
}

#[test]
fn cli_prints_agent_rules() {
    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["rules", "--agent"])
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# 지음 에이전트 규칙"));
    assert!(stdout.contains("truthiness를 도입하지 않는다"));
    assert!(stdout.contains("허용 메시지"));
    assert!(stdout.contains("점 표기를 도입하지 않는다"));
}

#[test]
fn cli_explains_diagnostic_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["explain", "MSG301"])
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MSG301"));
    assert!(stdout.contains("닫힌 메시지 집합"));
    assert!(stdout.contains("ziium rules --agent"));
}

#[test]
fn cli_check_json_reports_success() {
    let path = write_temp_program("이름은 \"철수\"이다");

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["check", "--json", path.to_str().expect("utf-8 path")])
        .output()
        .expect("cli should run");

    assert!(output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["ok"], true);
    assert_eq!(value["diagnostics"].as_array().unwrap().len(), 0);

    let _ = fs::remove_file(path);
}

#[test]
fn cli_check_json_reports_frontend_error() {
    let path = write_temp_program("이름은 \"철수\"이다\n이름을 출력해");

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["check", "--json", path.to_str().expect("utf-8 path")])
        .output()
        .expect("cli should run");

    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["ok"], false);
    assert_eq!(value["diagnostics"][0]["severity"], "error");
    assert_eq!(value["diagnostics"][0]["phase"], "parse");
    assert!(
        value["diagnostics"][0]["code"]
            .as_str()
            .unwrap()
            .starts_with("PAR")
    );
    assert!(
        value["diagnostics"][0]["expected"]
            .as_str()
            .unwrap()
            .contains("문장")
    );
    assert!(
        value["diagnostics"][0]["help"]
            .as_str()
            .unwrap()
            .contains("ziium explain")
    );

    let _ = fs::remove_file(path);
}

#[test]
fn cli_check_json_reports_resolve_error() {
    let path = write_temp_program("없는값을 출력한다");

    let output = Command::new(env!("CARGO_BIN_EXE_ziium"))
        .args(["check", "--json", path.to_str().expect("utf-8 path")])
        .output()
        .expect("cli should run");

    assert!(!output.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be json");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["ok"], false);
    assert_eq!(value["diagnostics"][0]["phase"], "resolve");
    assert_eq!(value["diagnostics"][0]["code"], "NAM003");
    assert!(value["diagnostics"][0]["message"]
        .as_str()
        .unwrap()
        .contains("아직 정의되지 않았습니다"));

    let _ = fs::remove_file(path);
}
