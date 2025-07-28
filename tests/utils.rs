use rand::Rng;
use std::{env, fs, path::PathBuf, process::Command, sync::Once};

static BUILD: Once = Once::new();

pub fn build() {
    BUILD.call_once(|| {
        assert!(Command::new("cargo")
            .arg("build")
            .output()
            .expect("failed to build extension")
            .status
            .success());
    });
}

pub fn write_test_file(script_name: &str, code: &str) -> PathBuf {
    let script_filename = env::current_dir()
        .unwrap()
        .join("tests/temp")
        .join(script_name);
    fs::write(script_filename.clone(), code).unwrap();
    script_filename
}

pub fn run_cli(code: &str) -> String {
    let rand_name = rand::thread_rng().gen_range(1..99999999).to_string() + ".php";
    let script_name = rand_name.as_str();
    let script_filename = write_test_file(&script_name, code);

    let output = Command::new("php")
        .arg(format!(
            "-dextension={}/target/debug/liberror_message_format.{}",
            env::current_dir().unwrap().to_str().unwrap(),
            std::env::consts::DLL_EXTENSION
        ))
        .arg("-c")
        .arg(env::current_dir().unwrap().join("tests/php.ini"))
        .arg(script_filename.clone())
        .output()
        .unwrap();

    dbg!(&output);

    fs::remove_file(script_filename).unwrap();
    String::from_utf8(output.stdout).unwrap()
}
