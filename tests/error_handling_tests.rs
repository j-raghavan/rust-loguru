use rust_loguru::error::*;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
struct TestError(&'static str);
impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError: {}", self.0)
    }
}
impl StdError for TestError {}

#[test]
fn test_result_ext_log_error() {
    let res: Result<(), TestError> = Err(TestError("fail"));
    let _ = res.log_error("Something went wrong");
}

#[test]
fn test_result_ext_with_context() {
    let res: Result<(), TestError> = Err(TestError("fail"));
    let res2 = res.with_context(|| "context info");
    match res2 {
        Err(e) => {
            assert_eq!(e.context, "context info");
            assert_eq!(e.error.0, "fail");
        }
        _ => panic!("Expected error"),
    }
}

#[test]
fn test_option_ext_log_none() {
    let opt: Option<u32> = None;
    let _ = opt.log_none("Missing value");
}

#[test]
fn test_error_chain() {
    let base = TestError("base");
    let ctx = ContextError {
        error: base,
        context: "ctx",
    };
    let chain = error_chain(&ctx);
    assert!(chain[0].contains("context: ctx"));
    assert!(chain[1].contains("TestError: base"));
}

#[test]
fn test_panic_hook_installation() {
    install_panic_hook();
    // We can't easily test actual panic output, but we can check that it doesn't panic to install twice
    install_panic_hook();
}

#[test]
fn test_source_location_macro() {
    let (file, line, col) = rust_loguru::source_location!();
    assert!(file.ends_with("error_handling_tests.rs"));
    assert!(line > 0);
    assert!(col > 0);
}

#[test]
fn test_log_error_with_location_macro() {
    let err = TestError("macro");
    rust_loguru::log_error_with_location!(err);
    rust_loguru::log_error_with_location!(err, "extra");
}
