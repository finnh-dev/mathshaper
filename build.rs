use std::fs;

fn main() {
    let env_path = ".env";
    let content = fs::read_to_string(env_path).expect("Failed to read .env file");

    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
}
