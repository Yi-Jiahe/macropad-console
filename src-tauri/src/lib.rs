use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

use active_win_pos_rs::get_active_window;

const VENDOR_ID: u16 = 0x10c4; // Arduino vendor ID
const PRODUCT_ID: u16 = 0xea60; // Arduino product ID

#[derive(Default)]
struct AppState {
    current_window: CurrentWindow,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentWindow {
    title: String,
    app_name: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));

            let handle = app.handle().clone();

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
