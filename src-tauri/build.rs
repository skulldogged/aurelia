use std::fs;
use std::process::Command;

fn main() {
    // Get the short commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to get git commit hash");

    let hash = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 in git output")
        .trim()
        .to_string();

    let version = format!("0.1.0-unstable.{}", hash);
    let cargo_path = "Cargo.toml";
    let original_cargo = fs::read_to_string(cargo_path).expect("Failed to read Cargo.toml");

    let tauri_path = "tauri.conf.json";
    let original_tauri = fs::read_to_string(tauri_path).expect("Failed to read tauri.conf.json");

    let updated_cargo = original_cargo
        .lines()
        .map(|line| {
            if line.trim().starts_with("version = ") {
                format!("version = \"{}\"", version)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(cargo_path, updated_cargo).expect("Failed to write Cargo.toml");

    let updated_tauri = original_tauri
        .lines()
        .map(|line| {
            if line.trim().starts_with("\"version\": ") {
                format!("  \"version\": \"{}\",", version)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(tauri_path, updated_tauri).expect("Failed to write tauri.conf.json");

    tauri_build::build();

    fs::write(cargo_path, original_cargo).expect("Failed to restore Cargo.toml");
    fs::write(tauri_path, original_tauri).expect("Failed to restore tauri.conf.json");
}
