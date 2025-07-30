use assertables::assert_contains;
use std::sync::{Arc, Condvar, Mutex, OnceLock};

mod utils;

// Global state to track build completion
static BUILD_STATE: OnceLock<(Arc<Mutex<bool>>, Arc<Condvar>)> = OnceLock::new();

// Call this function to ensure the build has happened and completed
// All threads will wait for the build to finish before proceeding
fn ensure_setup() {
    let (build_complete, condvar) = BUILD_STATE.get_or_init(|| {
        let pair = (Arc::new(Mutex::new(false)), Arc::new(Condvar::new()));
        let (build_complete, condvar) = pair.clone();

        // Start the build in the background
        std::thread::spawn(move || {
            utils::build();
            let mut completed = build_complete.lock().unwrap();
            *completed = true;
            condvar.notify_all();
        });

        pair
    });

    // Wait for build to complete
    let mut completed = build_complete.lock().unwrap();
    while !*completed {
        completed = condvar.wait(completed).unwrap();
    }
}

#[test]
fn test_build() {
    ensure_setup();
}

#[test]
fn test_version() {
    ensure_setup();
    let output = utils::run_cli("<?php phpinfo(); ?>");
    assert_contains!(
        output.trim(),
        &format!(
            "Error Message Format Version => {}",
            env!("CARGO_PKG_VERSION")
        )
    );
}

#[test]
fn test_cli_error_output_default() {
    ensure_setup();
    let code = r#"
<?php
trigger_error('This is a test error', E_USER_WARNING);
"#;
    let output = utils::run_cli(code);
    assert_contains!(output.trim(), "Warning: This is a test error in ");
}

#[test]
fn test_cli_error_output() {
    ensure_setup();
    let code = r#"
<?php
ini_set('error_message_format', '{message} with an append');
trigger_error('This is a test error', E_USER_WARNING);
"#;
    let output = utils::run_cli(code);
    assert_contains!(
        output.trim(),
        "Warning: This is a test error with an append in "
    );
}
