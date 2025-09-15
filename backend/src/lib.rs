use portpicker::pick_unused_port;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Write;
use tauri::{Manager, State};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Youve been greeted from Rust!", name)
}

// Struct to hold the port in Tauri's state
struct SidecarPort(u16);

#[derive(Deserialize)]
struct GreetResponse {
    response: String,
}

#[tauri::command]
async fn greet_python_proxy(
    name: String,
    port_state: State<'_, SidecarPort>,
) -> Result<String, String> {
    let port = port_state.0;
    let url = format!("http://127.0.0.1:{}/api/greet", port);

    let mut request_body = HashMap::new();
    request_body.insert("name", name);

    // FIXME: does tauri provide an HTTP client?
    let client = reqwest::Client::new();
    match client.post(&url).json(&request_body).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GreetResponse>().await {
                    Ok(greet_resp) => Ok(greet_resp.response),
                    Err(e) => Err(format!("Failed to parse JSON response from sidecar: {}", e)),
                }
            } else {
                Err(format!(
                    "Sidecar request failed with status: {}",
                    response.status()
                ))
            }
        }
        Err(e) => Err(format!("Failed to send request to sidecar: {}", report(&e))),
    }
}

fn report(mut err: &dyn std::error::Error) -> String {
    let mut s = format!("{}", err);
    while let Some(src) = err.source() {
        let _ = write!(s, "\n\nCaused by: {}", src);
        err = src;
    }
    s
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let port = 8000; //pick_unused_port().ok_or("no free port")?;

            let cmd = app
                .shell()
                .sidecar("main")
                .expect("failed to find sidecar called 'main'")
                .args(["--port", &port.to_string()]);

            let (mut rx, _) = cmd.spawn()?;

            tauri::async_runtime::spawn(async move {
                // read events such as stdout
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line_bytes) => {
                            let line = String::from_utf8_lossy(&line_bytes);
                            println!("{}", line);
                        }
                        CommandEvent::Stderr(line_bytes) => {
                            let line = String::from_utf8_lossy(&line_bytes);
                            println!("{}", line);
                        }
                        _ => {}
                    }
                }
            });
            app.manage(SidecarPort(port));

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, greet_python_proxy]) // Add new handler
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
