use std::fs;
use std::path::Path;

#[test]
fn parser_case_document_contains_expected_seed_cases() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("PARSER_TEST_CASES.md");
    let doc = fs::read_to_string(path).expect("parser case document should exist");

    let success_cases = doc.matches("### P").count();
    let failure_cases = doc.matches("### N").count();

    assert!(success_cases >= 30, "expected at least 30 success cases");
    assert!(failure_cases >= 10, "expected at least 10 failure cases");
    assert!(doc.contains("더하기 함수는 왼쪽, 오른쪽을 받아"));
    assert!(doc.contains("시작 함수는 아무것도 받지 않아"));
}
