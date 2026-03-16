use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::process::ExitCode;
use unicode_width::UnicodeWidthStr;
use ziium::{
    FrontendError, InterpreterSession, LexError, ParseError, ResolveError, RunError, RuntimeError,
    Span, Token, TokenKind, lex, parse_source, run_source,
};

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let first = args.next();
    let stdin_is_terminal = io::stdin().is_terminal();

    let (mode, path) = match first.as_deref() {
        None if stdin_is_terminal => ("repl", None),
        None => ("run", None),
        Some("run") | Some("tokens") | Some("ast") | Some("repl") => {
            (first.as_deref().unwrap(), args.next())
        }
        Some(path) => ("run", Some(path.to_string())),
    };

    if args.next().is_some() {
        return Err(usage());
    }

    match mode {
        "repl" => {
            if path.is_some() {
                return Err(usage());
            }
            run_repl()?;
        }
        "run" => {
            let source = read_source(path.as_deref()).map_err(render_input_error)?;
            let result = run_source(&source).map_err(|err| render_run_diagnostic(err, &source))?;
            for line in result.output {
                println!("{line}");
            }
        }
        "tokens" => {
            let source = read_source(path.as_deref()).map_err(render_input_error)?;
            let tokens = lex(&source).map_err(|err| render_lex_diagnostic(err, &source))?;
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
        _ => return Err(usage()),
    }

    Ok(())
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

fn run_repl_interactive() -> Result<(), String> {
    let mut editor = DefaultEditor::new().map_err(|err| {
        render_cli_error(
            "REPL 오류",
            format!("REPL 편집기를 시작하지 못했습니다: {err}"),
        )
    })?;
    let mut session = InterpreterSession::new();
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

    Ok(())
}

fn run_repl_stream() -> Result<(), String> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut session = InterpreterSession::new();
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
    for line in result.output {
        writeln!(stdout, "{line}").map_err(|err| {
            render_cli_error("REPL 오류", format!("REPL 출력을 쓰지 못했습니다: {err}"))
        })?;
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
    ["이면", "아니면", "동안", "받아", "않아"]
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

fn usage() -> String {
    render_cli_error("사용법", "ziium [run|tokens|ast|repl] [파일경로]")
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
