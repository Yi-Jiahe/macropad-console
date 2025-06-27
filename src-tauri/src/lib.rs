use std::collections::HashMap;
use std::sync::Mutex;

use active_win_pos_rs::get_active_window;
use anyhow::Result;
use dirs::home_dir;
use enigo::{Direction, Enigo, Key, Keyboard};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};

pub mod config;
pub mod hid;
use crate::config::{AppConfig, ApplicationProfile};
use crate::hid::{VENDOR_ID, PRODUCT_ID, USAGE_PAGE, USAGE};

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentWindow {
    title: String,
    app_name: String,
}

#[tauri::command]
fn get_config(state: State<'_, Mutex<AppConfig>>) -> String {
    let state = state.lock().unwrap();
    serde_json::to_string(&*state).unwrap()
}

#[tauri::command]
fn save_config(state: State<'_, Mutex<AppConfig>>, config_json: String) {
    println!("Saving config: {}", config_json);
    let mut state = state.lock().unwrap();
    *state = serde_json::from_str(&config_json).unwrap();

    let config_path = get_config_path();
    std::fs::write(config_path, config_json).unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(CurrentWindow::default()));
            app.manage(Mutex::new(AppConfig::default()));

            let handle = app.handle().clone();

            let config = match load_config() {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to load config: {}", e);
                    AppConfig::default()
                }
            };

            let state_app_config = handle.state::<Mutex<AppConfig>>();
            let mut state_app_config = state_app_config.lock().unwrap();
            *state_app_config = config;

            let window_tracker_handle = handle.clone();
            std::thread::spawn(move || {
                track_active_window(&window_tracker_handle);
            });

            let serial_handle = handle.clone();
            std::thread::spawn(move || {
                listen_hid(&serial_handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, save_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_config_path() -> std::path::PathBuf {
    home_dir()
        .unwrap()
        .join(".macropad-console")
        .join("config.json")
}

fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path();
    dbg!(&config_path);
    let config = std::fs::read_to_string(config_path)?;
    Ok(serde_json::from_str(&config)?)
}

fn track_active_window(handle: &tauri::AppHandle) {
    loop {
        if let Ok(active_window) = get_active_window() {
            let current_window = CurrentWindow {
                title: active_window.title,
                app_name: active_window.app_name,
            };

            // TODO: Skip update if no change

            // Update the current window
            let state_current_window = handle.state::<Mutex<CurrentWindow>>();
            let mut state_current_window = state_current_window.lock().unwrap();
            *state_current_window = current_window.clone();

            // Emit an event to notify the frontend
            handle
                .emit("active-window-changed", current_window)
                .unwrap();
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn listen_hid(handle: &tauri::AppHandle) {
    loop {
        let api = hidapi::HidApi::new().unwrap();
        // Print out information about all connected devices
        for device_info in api.device_list() {
            if device_info.vendor_id() == VENDOR_ID
                && device_info.product_id() == PRODUCT_ID
                && device_info.usage_page() == USAGE_PAGE
                && device_info.usage() == USAGE
            {
                if let Ok(device) = device_info.open_device(&api) {
                    device.set_blocking_mode(false).unwrap();

                    let mut buf: [u8; 1] = [0u8; 1]; // Buffer to hold the incoming data
                    loop {
                        match device.read(&mut buf[..]) {
                            Ok(0) => {
                                // No data read
                                // Sleep for a short duration to avoid busy-waiting
                                std::thread::sleep(std::time::Duration::from_millis(1));
                            }
                            Ok(n_bytes) => {
                                println!("Read: {:?}", &buf[..n_bytes]);

                                let state_app_config = handle.state::<Mutex<AppConfig>>();
                                let state_app_config = state_app_config.lock().unwrap();
                                let state_current_window = handle.state::<Mutex<CurrentWindow>>();
                                let state_current_window = state_current_window.lock().unwrap();

                                handle_report(
                                    state_app_config.application_profiles.clone(),
                                    state_current_window.app_name.clone(),
                                    buf,
                                );
                            }
                            Err(e) => {
                                // TODO: Continue on recoverable error, break on unrecoverable error, e.g disconnected device
                                eprintln!(
                                    "Failed to read from device: VID: 0x{:04x}, PID: 0x{:04x}, Error: {}",
                                    device_info.vendor_id(),
                                    device_info.product_id(),
                                    e
                                );
                                break;
                            }
                        }
                    }
                } else {
                    eprintln!(
                        "Failed to open device: VID: 0x{:04x}, PID: 0x{:04x}",
                        device_info.vendor_id(),
                        device_info.product_id()
                    );
                    continue;
                }
            }
        }
    }
}

fn handle_report(
    application_profiles: HashMap<String, ApplicationProfile>,
    app_name: String,
    report: [u8; 1],
) {
    if let Some(application_profile) = application_profiles.get(&app_name) {
        let buttons = report;

        let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();

        if let Some(encoder_config) = &application_profile.encoder {
            // if encoder_count != 0 {
            //     dbg!(encoder_count);

            //     let times = ((encoder_count as f32) * encoder_config.sensitivity)
            //         .abs()
            //         .floor() as i32;
            //     dbg!(times);

            //     let key = if encoder_count > 0 {
            //         encoder_config.up
            //     } else {
            //         encoder_config.down
            //     };

            //     for _ in 1..=times {
            //         enigo.key(Key::Unicode(key), Direction::Click).unwrap();
            //     }
            // }
        }
    }
}
