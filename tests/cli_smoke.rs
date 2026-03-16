use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_program(contents: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should work")
        .as_nanos();
    let path = env::temp_dir().join(format!("ziium_cli_{unique}.zm"));
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
