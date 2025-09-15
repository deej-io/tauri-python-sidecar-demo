use std::{fs::copy, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=../sidecar/main.py");

    Command::new("uv")
        .args(["run", "pyinstaller", "main.py"])
        .current_dir("../sidecar")
        .output()
        .expect("failed to bundle python sidecar");

    copy(
        "../sidecar/dist/main/main",
        format!(
            "../sidecar/dist/main/main-{}",
            std::env::var("TARGET").unwrap()
        ),
    )
    .expect("failed to rename built sidecar");

    tauri_build::build()
}
