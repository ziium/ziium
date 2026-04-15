use std::fs;

fn run_sample(path: &str) {
    let source =
        fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
    ziium::run_source(&source).unwrap_or_else(|err| panic!("{path} failed: {err}"));
}

#[test]
fn sample_00_hello_everyone() {
    run_sample("samples/00_hello_everyone.zm");
}
#[test]
fn sample_01_hello_name() {
    run_sample("samples/01_hello_name.zm");
}
#[test]
fn sample_02_if_else() {
    run_sample("samples/02_if_else.zm");
}
#[test]
fn sample_03_countdown() {
    run_sample("samples/03_countdown.zm");
}
#[test]
fn sample_04_add_function() {
    run_sample("samples/04_add_function.zm");
}
#[test]
fn sample_05_list_and_length() {
    run_sample("samples/05_list_and_length.zm");
}
#[test]
fn sample_06_record_profile() {
    run_sample("samples/06_record_profile.zm");
}
#[test]
fn sample_07_nested_properties() {
    run_sample("samples/07_nested_properties.zm");
}
#[test]
fn sample_08_message_syntax() {
    run_sample("samples/08_message_syntax.zm");
}
#[test]
fn sample_09_call_frames() {
    run_sample("samples/09_call_frames.zm");
}
#[test]
fn sample_10_hanoi() {
    run_sample("samples/10_hanoi.zm");
}
#[test]
fn sample_11_hanoi_ziium_style() {
    run_sample("samples/11_hanoi_ziium_style.zm");
}
#[test]
fn sample_12_canvas_hanoi() {
    run_sample("samples/12_canvas_hanoi.zm");
}
#[test]
fn sample_13_story() {
    run_sample("samples/13_story.zm");
}
#[test]
fn sample_14_foreach() {
    run_sample("samples/14_foreach.zm");
}
#[test]
fn sample_15_exist_binding() {
    run_sample("samples/15_exist_binding.zm");
}
#[test]
fn sample_16_type_conversion() {
    run_sample("samples/16_type_conversion.zm");
}
#[test]
fn sample_17_mutable_vs_const() {
    run_sample("samples/17_mutable_vs_const.zm");
}
#[test]
fn sample_18_inline_if_else() {
    run_sample("samples/18_inline_if_else.zm");
}
