use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::process::ExitCode;
use std::thread;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;
use ziium::{
    FrontendError, InterpreterSession, LexError, ParseError, ResolveError, RunError, RuntimeError,
    Span, Token, TokenKind, Value, lex, lex_ja, parse_source, parse_source_to_hir, parse_tokens,
    resolve_hir_program, resolve_program, with_particle_hint,
};

fn cli_choose(options: &[Value]) -> Result<Value, RuntimeError> {
    let rendered: Vec<String> = options.iter().map(|v| v.render()).collect();
    let mut stderr = io::stderr().lock();
    for (i, text) in rendered.iter().enumerate() {
        let _ = writeln!(stderr, "  {}) {text}", i + 1);
    }
    let _ = write!(stderr, "선택> ");
    let _ = stderr.flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| RuntimeError::new("입력을 읽을 수 없습니다."))?;
    let input = input.trim();

    // 번호로 선택
    if let Ok(n) = input.parse::<usize>()
        && n >= 1
        && n <= options.len()
    {
        return Ok(options[n - 1].clone());
    }

    // 텍스트로 선택
    for (i, text) in rendered.iter().enumerate() {
        if text == input {
            return Ok(options[i].clone());
        }
    }

    Err(RuntimeError::new(format!("잘못된 선택입니다: `{input}`")))
}

fn main() -> ExitCode {
    match run_cli() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> Result<ExitCode, String> {
    let mut args = env::args().skip(1);
    let first = args.next();
    let stdin_is_terminal = io::stdin().is_terminal();

    match first.as_deref() {
        Some("--version" | "-v") => {
            println!("ziium {}", env!("CARGO_PKG_VERSION"));
            return Ok(ExitCode::SUCCESS);
        }
        Some("--help" | "-h") => {
            println!("사용법: ziium [명령] [파일경로]");
            println!();
            println!("명령:");
            println!("  run     프로그램을 실행합니다 (기본값)");
            println!("  check   프로그램을 검사합니다 (--json 지원)");
            println!("  tokens  토큰 목록을 출력합니다");
            println!("  ast     구문 트리를 출력합니다");
            println!("  hir     HIR을 출력합니다");
            println!("  rules   에이전트용 언어 규칙을 출력합니다");
            println!("  explain 진단 코드를 설명합니다");
            println!("  repl    대화형 모드를 시작합니다");
            return Ok(ExitCode::SUCCESS);
        }
        Some("rules") => {
            let arg = args.next();
            if arg.as_deref() != Some("--agent") || args.next().is_some() {
                return Err(usage());
            }
            println!("{}", agent_rules());
            return Ok(ExitCode::SUCCESS);
        }
        Some("explain") => {
            let code = args.next().ok_or_else(usage)?;
            if args.next().is_some() {
                return Err(usage());
            }
            println!("{}", explain_diagnostic(&code)?);
            return Ok(ExitCode::SUCCESS);
        }
        _ => {}
    }

    let (mode, path, json) = match first.as_deref() {
        None if stdin_is_terminal => ("repl", None, false),
        None => ("run", None, false),
        Some("run") | Some("check") | Some("tokens") | Some("ast") | Some("hir") | Some("repl") => {
            let mode = first.as_deref().unwrap();
            let mut path = None;
            let mut json = false;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if path.is_none() {
                    path = Some(arg);
                } else {
                    return Err(usage());
                }
            }
            (mode, path, json)
        }
        Some(path) => ("run", Some(path.to_string()), false),
    };

    if args.next().is_some() {
        return Err(usage());
    }

    if json && mode != "check" {
        return Err(usage());
    }

    let lang = detect_language(path.as_deref());

    match mode {
        "repl" => {
            if path.is_some() {
                return Err(usage());
            }
            run_repl()?;
        }
        "check" => {
            let source = read_source(path.as_deref()).map_err(render_input_error)?;
            if json {
                let ok = run_check_json(&source, lang)?;
                return Ok(if ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                });
            }
            run_check(&source, lang).map_err(|err| render_frontend_diagnostic(err, &source))?;
            println!("검사 성공");
        }
        "run" => {
            let source = read_source(path.as_deref()).map_err(render_input_error)?;
            if lang == "ja" {
                let tokens = lex_ja(&source).map_err(|err| render_lex_diagnostic(err, &source))?;
                let program = parse_tokens(tokens)
                    .map_err(|err| render_frontend_diagnostic(err.into(), &source))?;
                let mut session = InterpreterSession::new();
                session.set_choose_fn(cli_choose);
                let result = session
                    .interpret_program(&program)
                    .map_err(|err| render_run_diagnostic(RunError::Runtime(err), &source))?;
                print_output(result)?;
            } else {
                let mut session = InterpreterSession::new();
                session.set_choose_fn(cli_choose);
                let result = session
                    .run_source(&source)
                    .map_err(|err| render_run_diagnostic(err, &source))?;
                print_output(result)?;
            }
        }
        "tokens" => {
            let source = read_source(path.as_deref()).map_err(render_input_error)?;
            let tokens = if lang == "ja" {
                lex_ja(&source).map_err(|err| render_lex_diagnostic(err, &source))?
            } else {
                lex(&source).map_err(|err| render_lex_diagnostic(err, &source))?
            };
            for token in tokens {
                println!("{}", render_token(&token));
            }
        }
        "ast" => {
            let source = read_source(path.as_deref()).map_err(render_input_error)?;
            let program =
                parse_source(&source).map_err(|err| render_frontend_diagnostic(err, &source))?;
            println!("{program:#?}");
        }
        "hir" => {
            let source = read_source(path.as_deref()).map_err(render_input_error)?;
            let program = parse_source_to_hir(&source)
                .map_err(|err| render_frontend_diagnostic(err, &source))?;
            println!("{program:#?}");
        }
        _ => return Err(usage()),
    }

    Ok(ExitCode::SUCCESS)
}

fn run_check(source: &str, lang: &str) -> Result<(), FrontendError> {
    if lang == "ja" {
        let tokens = lex_ja(source).map_err(FrontendError::Lex)?;
        let program = parse_tokens(tokens).map_err(FrontendError::Parse)?;
        resolve_program(&program).map_err(FrontendError::Resolve)?;
    } else {
        let program = parse_source_to_hir(source)?;
        resolve_hir_program(&program)
            .map_err(|err| with_particle_hint(err, source))
            .map_err(FrontendError::Resolve)?;
    }
    Ok(())
}

fn run_check_json(source: &str, lang: &str) -> Result<bool, String> {
    let result = match run_check(source, lang) {
        Ok(()) => CheckJson {
            schema_version: 1,
            ok: true,
            diagnostics: Vec::new(),
        },
        Err(err) => CheckJson {
            schema_version: 1,
            ok: false,
            diagnostics: vec![DiagnosticJson::from_frontend_error(&err)],
        },
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&result).map_err(|err| render_cli_error(
            "JSON 오류",
            format!("JSON을 만들지 못했습니다: {err}")
        ))?
    );
    Ok(result.ok)
}

fn read_source(path: Option<&str>) -> io::Result<String> {
    match path {
        Some(path) => fs::read_to_string(path),
        None => {
            let mut source = String::new();
            io::stdin().read_to_string(&mut source)?;
            Ok(source)
        }
    }
}

fn render_token(token: &Token) -> String {
    match token.kind {
        TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => {
            format!("{:?}", token.kind)
        }
        _ => format!("{:?}({:?})", token.kind, token.lexeme),
    }
}

fn run_repl() -> Result<(), String> {
    if io::stdin().is_terminal() {
        return run_repl_interactive();
    }

    run_repl_stream()
}

fn repl_history_path() -> Option<std::path::PathBuf> {
    env::var("HOME").ok().map(|home| {
        let dir = std::path::PathBuf::from(home).join(".ziium");
        let _ = fs::create_dir_all(&dir);
        dir.join("history")
    })
}

fn run_repl_interactive() -> Result<(), String> {
    let mut editor = DefaultEditor::new().map_err(|err| {
        render_cli_error(
            "REPL 오류",
            format!("REPL 편집기를 시작하지 못했습니다: {err}"),
        )
    })?;
    let history_path = repl_history_path();
    if let Some(ref path) = history_path {
        let _ = editor.load_history(path);
    }
    let mut session = InterpreterSession::new();
    session.set_choose_fn(cli_choose);
    let mut buffer = Vec::new();

    loop {
        match editor.readline(repl_prompt(buffer.is_empty())) {
            Ok(line) => {
                if !line.trim().is_empty() {
                    let _ = editor.add_history_entry(line.as_str());
                }

                if matches!(
                    process_repl_line(&mut session, &mut buffer, line)?,
                    ReplLoopAction::Exit
                ) {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                if buffer.is_empty() {
                    eprintln!("입력이 취소되었습니다.");
                } else {
                    buffer.clear();
                    eprintln!("현재 입력 중인 블록을 취소했습니다.");
                }
            }
            Err(ReadlineError::Eof) => {
                if matches!(
                    finish_repl_input(&mut session, &mut buffer)?,
                    ReplLoopAction::Exit
                ) {
                    break;
                }
            }
            Err(err) => {
                return Err(render_cli_error(
                    "REPL 오류",
                    format!("REPL 입력을 읽지 못했습니다: {err}"),
                ));
            }
        }
    }

    if let Some(ref path) = history_path {
        let _ = editor.save_history(path);
    }

    Ok(())
}

fn run_repl_stream() -> Result<(), String> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut session = InterpreterSession::new();
    session.set_choose_fn(cli_choose);
    let mut buffer = Vec::new();

    loop {
        print_prompt(buffer.is_empty())?;

        let mut line = String::new();
        let read = input.read_line(&mut line).map_err(|err| {
            render_cli_error("REPL 오류", format!("REPL 입력을 읽지 못했습니다: {err}"))
        })?;

        if read == 0 {
            if matches!(
                finish_repl_input(&mut session, &mut buffer)?,
                ReplLoopAction::Exit
            ) {
                break;
            }
            continue;
        }

        let line = line.trim_end_matches(['\n', '\r']).to_string();
        if matches!(
            process_repl_line(&mut session, &mut buffer, line)?,
            ReplLoopAction::Exit
        ) {
            break;
        }
    }

    Ok(())
}

fn process_repl_line(
    session: &mut InterpreterSession,
    buffer: &mut Vec<String>,
    line: String,
) -> Result<ReplLoopAction, String> {
    let trimmed = line.trim();
    let opens_block = line_opens_block(trimmed);

    if trimmed.starts_with(':') {
        return match handle_repl_command(trimmed, buffer)? {
            ReplCommand::Continue => Ok(ReplLoopAction::Continue),
            ReplCommand::Exit => Ok(ReplLoopAction::Exit),
        };
    }

    if trimmed.is_empty() {
        if buffer.is_empty() {
            return Ok(ReplLoopAction::Continue);
        }

        return match evaluate_repl_buffer(session, &buffer.join("\n")) {
            ReplAction::Run(result) => {
                print_output(result)?;
                buffer.clear();
                Ok(ReplLoopAction::Continue)
            }
            ReplAction::Wait => {
                eprintln!("입력이 아직 끝나지 않았습니다. `:reset`으로 취소할 수 있습니다.");
                Ok(ReplLoopAction::Continue)
            }
            ReplAction::Error(message) => {
                eprintln!("{message}");
                buffer.clear();
                Ok(ReplLoopAction::Continue)
            }
        };
    }

    buffer.push(line);
    if buffer_requires_explicit_submit(&buffer.join("\n")) {
        if opens_block {
            print_block_input_guide()?;
        }
        return Ok(ReplLoopAction::Continue);
    }

    match evaluate_repl_buffer(session, &buffer.join("\n")) {
        ReplAction::Run(result) => {
            print_output(result)?;
            buffer.clear();
        }
        ReplAction::Wait => {}
        ReplAction::Error(message) => {
            eprintln!("{message}");
            buffer.clear();
        }
    }

    Ok(ReplLoopAction::Continue)
}

fn finish_repl_input(
    session: &mut InterpreterSession,
    buffer: &mut Vec<String>,
) -> Result<ReplLoopAction, String> {
    if buffer.is_empty() {
        return Ok(ReplLoopAction::Exit);
    }

    match evaluate_repl_buffer(session, &buffer.join("\n")) {
        ReplAction::Run(result) => {
            print_output(result)?;
            buffer.clear();
            Ok(ReplLoopAction::Exit)
        }
        ReplAction::Wait => Err("입력이 아직 끝나지 않았습니다.".to_string()),
        ReplAction::Error(message) => Err(message),
    }
}

fn repl_prompt(is_fresh: bool) -> &'static str {
    if is_fresh { "ziium> " } else { "....> " }
}

fn print_prompt(is_fresh: bool) -> Result<(), String> {
    let prompt = repl_prompt(is_fresh);
    let mut stderr = io::stderr().lock();
    write!(stderr, "{prompt}").map_err(|err| {
        render_cli_error(
            "REPL 오류",
            format!("REPL 프롬프트를 출력하지 못했습니다: {err}"),
        )
    })?;
    stderr.flush().map_err(|err| {
        render_cli_error(
            "REPL 오류",
            format!("REPL 프롬프트를 비우지 못했습니다: {err}"),
        )
    })
}

fn print_output(result: ziium::ExecutionResult) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    for event in result.events {
        match event {
            ziium::ExecutionEvent::Output { text } => {
                writeln!(stdout, "{text}").map_err(|err| {
                    render_cli_error("REPL 오류", format!("REPL 출력을 쓰지 못했습니다: {err}"))
                })?;
            }
            ziium::ExecutionEvent::Sleep { seconds } => {
                thread::sleep(Duration::from_secs_f64(seconds));
            }
            ziium::ExecutionEvent::CanvasFrame { .. } => {}
        }
    }
    stdout.flush().map_err(|err| {
        render_cli_error("REPL 오류", format!("REPL 출력을 비우지 못했습니다: {err}"))
    })
}

fn print_block_input_guide() -> Result<(), String> {
    let mut stderr = io::stderr().lock();
    writeln!(
        stderr,
        "안내: 다음 줄부터 두 칸 들여써 블록을 입력하세요. 빈 줄을 입력하면 실행합니다."
    )
    .map_err(|err| {
        render_cli_error(
            "REPL 오류",
            format!("REPL 블록 안내를 출력하지 못했습니다: {err}"),
        )
    })?;
    stderr.flush().map_err(|err| {
        render_cli_error(
            "REPL 오류",
            format!("REPL 블록 안내를 비우지 못했습니다: {err}"),
        )
    })
}

fn handle_repl_command(command: &str, buffer: &mut Vec<String>) -> Result<ReplCommand, String> {
    match command {
        ":quit" | ":exit" => Ok(ReplCommand::Exit),
        ":reset" => {
            buffer.clear();
            Ok(ReplCommand::Continue)
        }
        ":help" => {
            let mut stderr = io::stderr().lock();
            writeln!(stderr, ":help   도움말을 출력합니다.")
                .and_then(|_| writeln!(stderr, ":reset  현재 입력 중인 블록을 취소합니다."))
                .and_then(|_| writeln!(stderr, ":quit   REPL을 종료합니다."))
                .map_err(|err| {
                    render_cli_error(
                        "REPL 오류",
                        format!("REPL 도움말을 출력하지 못했습니다: {err}"),
                    )
                })?;
            stderr.flush().map_err(|err| {
                render_cli_error(
                    "REPL 오류",
                    format!("REPL 도움말을 비우지 못했습니다: {err}"),
                )
            })?;
            Ok(ReplCommand::Continue)
        }
        _ => Err(render_cli_error(
            "REPL 오류",
            format!("알 수 없는 REPL 명령입니다: {command}"),
        )),
    }
}

fn evaluate_repl_buffer(session: &mut InterpreterSession, source: &str) -> ReplAction {
    match session.run_source(source) {
        Ok(result) => ReplAction::Run(result),
        Err(RunError::Frontend(err)) if needs_more_repl_input(source, &err) => ReplAction::Wait,
        Err(err) => ReplAction::Error(render_run_diagnostic(err, source)),
    }
}

fn buffer_requires_explicit_submit(source: &str) -> bool {
    last_significant_line(source).is_some_and(line_opens_block)
        || source.lines().any(|line| {
            line.chars()
                .next()
                .is_some_and(|ch| ch == ' ' || ch == '\t')
        })
}

fn needs_more_repl_input(source: &str, err: &FrontendError) -> bool {
    if has_unterminated_string(source) || has_unclosed_delimiters(source) {
        return true;
    }

    if last_significant_line(source).is_some_and(line_opens_block) {
        return true;
    }

    match err {
        FrontendError::Lex(LexError::UnterminatedString { .. }) => true,
        FrontendError::Parse(ParseError { message, .. }) => {
            message.contains("들여쓴 블록")
                || message == "표현식이 끝나지 않았습니다."
                || message.contains("닫혀야 합니다")
                || last_significant_line(source).is_some_and(line_looks_incomplete)
        }
        _ => false,
    }
}

fn last_significant_line(source: &str) -> Option<&str> {
    source.lines().rev().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn line_opens_block(line: &str) -> bool {
    [
        "이면",
        "아니면",
        "동안",
        "받아",
        "않아",
        "크면",
        "작으면",
        "같으면",
        "다르면",
    ]
    .into_iter()
    .any(|suffix| line.ends_with(suffix))
}

fn line_looks_incomplete(line: &str) -> bool {
    [
        "은", "는", "을", "를", "의", "로", "으로", ",", ":", "+", "-", "*", "/", "%",
    ]
    .into_iter()
    .any(|suffix| line.ends_with(suffix))
}

fn has_unclosed_delimiters(source: &str) -> bool {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut in_string = false;

    for line in source.lines() {
        for ch in line.chars() {
            if in_string {
                if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '#' => break,
                '"' => in_string = true,
                '(' => paren_depth += 1,
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
        }
    }

    in_string || paren_depth > 0 || bracket_depth > 0 || brace_depth > 0
}

fn has_unterminated_string(source: &str) -> bool {
    let mut in_string = false;

    for line in source.lines() {
        for ch in line.chars() {
            if in_string {
                if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '#' => break,
                '"' => in_string = true,
                _ => {}
            }
        }
    }

    in_string
}

enum ReplCommand {
    Continue,
    Exit,
}

enum ReplLoopAction {
    Continue,
    Exit,
}

enum ReplAction {
    Run(ziium::ExecutionResult),
    Wait,
    Error(String),
}

#[derive(Debug, Serialize)]
struct CheckJson {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    ok: bool,
    diagnostics: Vec<DiagnosticJson>,
}

#[derive(Debug, Serialize)]
struct DiagnosticJson {
    severity: &'static str,
    phase: &'static str,
    code: &'static str,
    message: String,
    line: Option<usize>,
    column: Option<usize>,
    length: Option<usize>,
    expected: &'static str,
    actual: String,
    help: String,
    #[serde(rename = "fixSafety")]
    fix_safety: &'static str,
    repair: RepairJson,
}

#[derive(Debug, Serialize)]
struct RepairJson {
    id: &'static str,
    summary: &'static str,
}

impl DiagnosticJson {
    fn from_frontend_error(err: &FrontendError) -> Self {
        let span = frontend_error_span(err);
        let (phase, code, message, expected, repair_id, repair_summary) = match err {
            FrontendError::Lex(err) => (
                "lex",
                lex_diagnostic_code(err),
                err.to_string(),
                "UTF-8 지음 소스, 공백 들여쓰기, 닫힌 문자열",
                "edit-lexical-form",
                "문자, 문자열, 들여쓰기 형태를 지음 어휘 규칙에 맞게 고칩니다.",
            ),
            FrontendError::Parse(err) => (
                "parse",
                parse_diagnostic_code(err),
                err.message.clone(),
                "닫힌 지음 문장 문법",
                "edit-syntax-form",
                "허용된 문장 프레임과 조사 위치에 맞게 문장을 고칩니다.",
            ),
            FrontendError::Resolve(err) => (
                "resolve",
                resolve_diagnostic_code(err),
                err.message.clone(),
                "현재 스코프의 이름, 가변성, 닫힌 built-in 메시지",
                "edit-name-or-message",
                "이름 선언, 가변 바인딩, 허용 메시지 집합을 확인합니다.",
            ),
        };

        Self {
            severity: "error",
            phase,
            code,
            message,
            line: span.map(|span| span.start_line),
            column: span.map(|span| span.start_column),
            length: span.map(diagnostic_span_len),
            expected,
            actual: frontend_actual(err),
            help: format!("ziium explain {code}"),
            fix_safety: "requires-human-review",
            repair: RepairJson {
                id: repair_id,
                summary: repair_summary,
            },
        }
    }
}

fn diagnostic_span_len(span: &Span) -> usize {
    if span.start_line == span.end_line && span.end_column > span.start_column {
        span.end_column.saturating_sub(span.start_column)
    } else {
        1
    }
}

fn frontend_actual(err: &FrontendError) -> String {
    match err {
        FrontendError::Lex(err) => match err {
            LexError::UnexpectedCharacter { ch, .. } => format!("예상하지 못한 문자 `{ch}`"),
            LexError::UnterminatedString { .. } => "닫히지 않은 문자열".to_string(),
            LexError::TabIndentation { .. } => "탭 들여쓰기".to_string(),
            LexError::InconsistentDedent { .. } => "맞지 않는 들여쓰기 깊이".to_string(),
        },
        FrontendError::Parse(err) => err.message.clone(),
        FrontendError::Resolve(err) => err.message.clone(),
    }
}

fn lex_diagnostic_code(err: &LexError) -> &'static str {
    match err {
        LexError::UnexpectedCharacter { .. } => "LEX100",
        LexError::UnterminatedString { .. } => "LEX101",
        LexError::TabIndentation { .. } => "LEX102",
        LexError::InconsistentDedent { .. } => "LEX103",
    }
}

fn parse_diagnostic_code(err: &ParseError) -> &'static str {
    if err.message.contains("메시지")
        || err.message.contains("추가")
        || err.message.contains("제곱")
        || err.message.contains("더하기")
    {
        "MSG301"
    } else if err.message.contains("블록") || err.message.contains("들여") {
        "PAR101"
    } else {
        "PAR100"
    }
}

fn resolve_diagnostic_code(err: &ResolveError) -> &'static str {
    if err.message.contains("메시지")
        || err.message.contains("길이")
        || err.message.contains("추가")
        || err.message.contains("제곱")
    {
        "MSG301"
    } else if err.message.contains("바꿀 수 없습니다") || err.message.contains("가변") {
        "NAM004"
    } else {
        "NAM003"
    }
}

fn detect_language(path: Option<&str>) -> &'static str {
    match path {
        Some(p) if p.ends_with(".zmj") => "ja",
        _ => "ko",
    }
}

fn usage() -> String {
    render_cli_error(
        "사용법",
        "ziium [run|tokens|ast|hir|repl] [파일경로]\n       ziium check [--json] [파일경로]\n       ziium rules --agent\n       ziium explain <진단코드>",
    )
}

fn agent_rules() -> &'static str {
    r#"# 지음 에이전트 규칙

## 핵심 원칙
- 지음은 영어 키워드의 한글 번역판이 아니다.
- 조사와 서술어를 장식으로 취급하지 않는다.
- surface syntax와 내부 표현을 섞지 않는다.
- 자연어 추론으로 parser를 확장하지 않는다.
- truthiness를 도입하지 않는다.

## 표면 문법
- 블록은 들여쓰기 기반이다. 중괄호나 `끝` 토큰을 쓰지 않는다.
- 함수 정의는 `<이름> 함수는` 형식을 쓴다.
- 속성 접근은 `의`를 쓴다. 점 표기를 도입하지 않는다.
- 기본 파일 확장자는 `.zm`이다.

## 허용 메시지
- `길이`
- `제곱`
- `더하기`, `빼기`, `곱하기`, `나누기`
- `추가`
- `지우기`, `점찍기`, `사각형채우기`, `글자쓰기`

## 메시지 경계
- `더하기`류는 infix 위치에서만 특별 취급한다.
- `추가`는 `추가(...)` 또는 `<목록>에 <값> 추가`에서만 특별 취급한다.
- `제곱`은 정수/실수의 `의` 프레임에서만 특별 취급한다.
- 결과 서술 문법은 현재 허용된 `꺼낸 것이다`/`꺼낸다` frame만 사용한다.

## 에이전트 작업 절차
- 코드를 만들기 전에 가까운 샘플을 먼저 확인한다.
- 오류가 나면 `ziium check --json <파일>`의 `code`, `expected`, `actual`, `help`를 기준으로 고친다.
- 새 문법을 열어야 하면 구현보다 먼저 `docs/LANGUAGE.md`, `docs/GRAMMAR.ebnf`, `docs/DECISIONS.md`를 갱신한다.
"#
}

fn explain_diagnostic(code: &str) -> Result<&'static str, String> {
    match code {
        "LEX100" => Ok(
            "LEX100: 예상하지 못한 문자\n\n지음 lexer가 현재 문맥에서 허용하지 않는 문자를 만났습니다. 문자열은 큰따옴표로 감싸고, 주석은 `#`으로 시작하며, 문장 기호는 현재 문법에 있는 것만 사용하세요.",
        ),
        "LEX101" => Ok(
            "LEX101: 닫히지 않은 문자열\n\n문자열 리터럴은 같은 줄에서 큰따옴표로 닫혀야 합니다.",
        ),
        "LEX102" => Ok(
            "LEX102: 탭 들여쓰기\n\n지음 블록은 공백 들여쓰기만 허용합니다. 탭을 공백으로 바꾸세요.",
        ),
        "LEX103" => Ok(
            "LEX103: 맞지 않는 들여쓰기 깊이\n\n블록을 닫는 줄의 들여쓰기 깊이가 이전 블록 경계와 일치해야 합니다.",
        ),
        "PAR100" => Ok(
            "PAR100: 구문 오류\n\n입력이 닫힌 지음 문장 문법과 맞지 않습니다. `ziium rules --agent`로 현재 허용 문장과 메시지 경계를 확인하세요.",
        ),
        "PAR101" => Ok(
            "PAR101: 블록 구문 오류\n\n`이면`, `아니면`, `인 동안`, `<함수> 함수는 ... 받아` 다음 줄은 공백으로 들여쓴 블록이어야 합니다.",
        ),
        "NAM003" => Ok(
            "NAM003: 이름 해석 오류\n\n현재 스코프에서 보이는 바인딩, 매개변수, 함수, built-in 이름만 사용할 수 있습니다.",
        ),
        "NAM004" => Ok(
            "NAM004: 이름 사용 형태 오류\n\n불변 바인딩을 `바꾼다`로 재대입하거나, 같은 스코프에서 이름을 중복 선언했을 가능성이 있습니다.",
        ),
        "MSG301" => Ok(
            "MSG301: 닫힌 메시지 집합 위반\n\n현재 메시지 집합은 built-in으로 닫혀 있습니다. 허용 메시지는 `길이`, `제곱`, `더하기/빼기/곱하기/나누기`, `추가`, `지우기`, `점찍기`, `사각형채우기`, `글자쓰기`입니다. 자세한 경계는 `ziium rules --agent`를 확인하세요.",
        ),
        _ => Err(render_cli_error(
            "진단 설명 오류",
            format!("알 수 없는 진단 코드입니다: {code}"),
        )),
    }
}

fn render_input_error(err: io::Error) -> String {
    render_cli_error("입력 오류", format!("입력을 읽지 못했습니다: {err}"))
}

fn render_cli_error(kind: &str, message: impl Into<String>) -> String {
    format!("[{kind}]\n메시지: {}", message.into())
}

fn render_run_diagnostic(err: RunError, source: &str) -> String {
    render_source_diagnostic(err.to_string(), run_error_span(&err), source)
}

fn render_frontend_diagnostic(err: FrontendError, source: &str) -> String {
    render_source_diagnostic(err.to_string(), frontend_error_span(&err), source)
}

fn render_lex_diagnostic(err: LexError, source: &str) -> String {
    render_source_diagnostic(err.to_string(), lex_error_span(&err), source)
}

fn render_source_diagnostic(message: String, span: Option<&Span>, source: &str) -> String {
    match span.and_then(|span| render_code_frame(source, span)) {
        Some(frame) => format!("{message}\n{frame}"),
        None => message,
    }
}

fn render_code_frame(source: &str, span: &Span) -> Option<String> {
    let line_number = span.start_line;
    let line_index = line_number.checked_sub(1)?;
    let lines = source.lines().collect::<Vec<_>>();
    let line = *lines.get(line_index)?;
    let number_width = line_number.to_string().len();
    let start_index = span.start_column.saturating_sub(1);
    let end_index = if span.start_line == span.end_line && span.end_column > span.start_column {
        span.end_column.saturating_sub(1)
    } else {
        span.start_column
    };

    let prefix = slice_chars(line, 0, start_index);
    let highlighted = slice_chars(line, start_index, end_index);
    let caret_padding = " ".repeat(UnicodeWidthStr::width(prefix.as_str()));
    let caret_width = UnicodeWidthStr::width(highlighted.as_str()).max(1);
    let mut frame_lines = vec!["코드:".to_string()];

    if let Some(previous_line) = line_index.checked_sub(1).and_then(|index| lines.get(index)) {
        frame_lines.push(render_code_frame_line(
            line_number - 1,
            previous_line,
            number_width,
        ));
    }

    frame_lines.push(render_code_frame_line(line_number, line, number_width));
    frame_lines.push(format!(
        "{} | {}{}",
        " ".repeat(number_width),
        caret_padding,
        "^".repeat(caret_width),
    ));

    if let Some(next_line) = lines.get(line_index + 1) {
        frame_lines.push(render_code_frame_line(
            line_number + 1,
            next_line,
            number_width,
        ));
    }

    Some(frame_lines.join("\n"))
}

fn render_code_frame_line(line_number: usize, line: &str, number_width: usize) -> String {
    format!("{line_number:>number_width$} | {line}")
}

fn slice_chars(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn run_error_span(err: &RunError) -> Option<&Span> {
    match err {
        RunError::Frontend(err) => frontend_error_span(err),
        RunError::Runtime(err) => runtime_error_span(err),
    }
}

fn frontend_error_span(err: &FrontendError) -> Option<&Span> {
    match err {
        FrontendError::Lex(err) => lex_error_span(err),
        FrontendError::Parse(err) => parse_error_span(err),
        FrontendError::Resolve(err) => resolve_error_span(err),
    }
}

fn lex_error_span(err: &LexError) -> Option<&Span> {
    match err {
        LexError::UnexpectedCharacter { span, .. }
        | LexError::UnterminatedString { span }
        | LexError::TabIndentation { span } => Some(span),
        LexError::InconsistentDedent { .. } => None,
    }
}

fn parse_error_span(err: &ParseError) -> Option<&Span> {
    err.span.as_ref()
}

fn resolve_error_span(err: &ResolveError) -> Option<&Span> {
    err.span.as_ref()
}

fn runtime_error_span(err: &RuntimeError) -> Option<&Span> {
    err.span.as_ref()
}
