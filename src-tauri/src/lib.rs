use std::sync::Mutex;
use std::collections::HashMap;

use active_win_pos_rs::get_active_window;
use dirs::home_dir;
use anyhow::Result;
use enigo::{Enigo, Key, Keyboard, Direction};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};


const VENDOR_ID: u16 = 0x10c4; // Arduino vendor ID
const PRODUCT_ID: u16 = 0xea60; // Arduino product ID

#[derive(Default)]
struct AppState {
    current_window: CurrentWindow,
    app_config: AppConfig,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentWindow {
    title: String,
    app_name: String,
}

type ApplicationProfile = HashMap<String, Vec<char>>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
    application_profiles: HashMap<String, ApplicationProfile>,
}

#[tauri::command]
fn get_config(state: State<'_, Mutex<AppState>>) -> String {
    let state = state.lock().unwrap();
    serde_json::to_string(&state.app_config).unwrap()
}

#[tauri::command]
fn save_config(state: State<'_, Mutex<AppState>>, config_json: String) {
    println!("Saving config: {}", config_json);
    let mut state = state.lock().unwrap();
    state.app_config = serde_json::from_str(&config_json).unwrap();

    let config_path = get_config_path();
    std::fs::write(config_path, config_json).unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));

            let handle = app.handle().clone();

            let config = match load_config() {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to load config: {}", e);
                    AppConfig::default()
                }
            };

            let state = handle.state::<Mutex<AppState>>();
            let mut state = state.lock().unwrap();
            state.app_config = config;

            let window_tracker_handle = handle.clone();
            std::thread::spawn(move || {
                track_active_window(&window_tracker_handle);
            });

            let serial_handle = handle.clone();
            std::thread::spawn(move || {
                listen_serial(&serial_handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_config_path() -> std::path::PathBuf {
    home_dir().unwrap().join(".macropad-console").join("config.json")    
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
            let state = handle.state::<Mutex<AppState>>();
            let mut state = state.lock().unwrap();
            state.current_window = current_window.clone();

            // Emit an event to notify the frontend
            handle
                .emit("active-window-changed", current_window)
                .unwrap();

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

fn find_serial_port(vendor_id: u16, product_id: u16) -> Option<String> {
    for port_info in &serialport::available_ports().unwrap() {
        dbg!(&port_info);
        if let serialport::SerialPortType::UsbPort(usb_port_info) = &port_info.port_type {
            if usb_port_info.vid == vendor_id && usb_port_info.pid == product_id {
                return Some(port_info.port_name.clone());
            }
        }
    }

    None
}

fn listen_serial(handle: &tauri::AppHandle) {
    loop {
        // Attempt to find a connected device
        println!("Searching for serial port...");
        if let Some(serial_port) = find_serial_port(VENDOR_ID, PRODUCT_ID) {
            let mut port = match serialport::new(&serial_port, 115200)
                .timeout(std::time::Duration::from_millis(100))
                .open()
            {
                Ok(port) => port,
                Err(e) => {
                    println!("Error opening serial port: {}", e);
                    continue;
                }
            };

            // TODO: Figure out a more reliable message structure
            let mut buf: Vec<u8> = vec![0; 1024]; // Buffer to hold the incoming data
            let mut message = String::new(); // String to hold the message
            loop {
                match port.read(&mut buf) {
                    Ok(t) => {
                        if t > 0 {
                            // Convert the bytes read into a string and append to the message
                            for i in 0..t {
                                let byte = buf[i];

                                // Check if the byte is a newline (end of message)
                                if byte == b'\n' {
                                    let state = handle.state::<Mutex<AppState>>();
                                    let state = state.lock().unwrap();
                                    handle_message(state.app_config.application_profiles.clone(), state.current_window.app_name.clone(), message.clone().trim().to_string());

                                    // Emit an event to notify the frontend
                                    handle.emit("serial-message", message.clone()).unwrap();

                                    // Clear the message
                                    message.clear();
                                }

                                // Otherwise, append the byte to the message
                                message.push(byte as char);
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::TimedOut {
                            // Do not allow any error that is not a timeout
                            println!("Error reading serial port: {}", e);
                            // Exit to outer loop to attempt to reconnect in order to fix connection issues
                            break;
                        }
                    }
                }
            }
        }

        // If no serial port is found, wait for 1 second before trying again
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn handle_message(application_profiles: HashMap<String, ApplicationProfile>, app_name: String, message: String) {
    if let Some(application_profile) = application_profiles.get(&app_name) {
        println!("Found application profile for app: {}", app_name);
        dbg!(&message);
        if let Some(keys) = application_profile.get(&message) {
            println!("Found keys for message: {}", message);
            // TODO: Hoist this up so that it doesn't need to be recreated
            let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();

            // TODO: Add support for modifiers, key up, key down, figure out how to express chars
            for key in keys {
                println!("Sending key: {}", key);
                enigo.key(Key::Unicode(*key), Direction::Press).unwrap();
            }

            for key in keys {
                println!("Sending key: {}", key);
                enigo.key(Key::Unicode(*key), Direction::Release).unwrap();
            }
        }
    }
}