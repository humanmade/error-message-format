use assertables::assert_contains;

mod utils;

#[test]
fn test_build() {
    utils::build();
}

#[test]
fn test_version() {
	utils::build();
	let output = utils::run_cli("<?php phpinfo(); ?>");
	assert_contains!(output.trim(), &format!("Error Message Format Version => {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_cli_error_output_default() {
    utils::build();
    let code = r#"
<?php
trigger_error('This is a test error', E_USER_WARNING);
"#;
    let output = utils::run_cli(code);
    assert_contains!(output.trim(), "Warning: This is a test error in ");
}

#[test]
fn test_cli_error_output() {
    utils::build();
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
