use std::collections::HashSet;
use std::sync::Mutex;

use anyhow::Result;
use enigo::{Direction, Enigo, Key, Keyboard, Mouse, Settings};
use regex::Regex;
use serde::Serialize;
use tauri::{Emitter, Manager, State};
use windows::{
    Win32::Foundation::HWND,
    Win32::System::ProcessStatus::K32GetModuleBaseNameW,
    Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    },
};

pub mod config;
pub mod events;
pub mod hid;
pub mod macropad_state;
use crate::config::{
    get_config_path, load_config, Action, AppConfig, ApplicationProfile, Command, Operation,
    RadialMenuItem,
};
use crate::hid::{handle_report, PRODUCT_ID, USAGE, USAGE_PAGE, VENDOR_ID};
use crate::macropad_state::{ButtonState, MacropadState};

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurrentWindow {
    title: String,
    app_name: String,
}

#[tauri::command]
fn get_config(state: State<'_, Mutex<AppConfig>>) -> String {
    let mut state = state.lock().unwrap();

    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            AppConfig::default()
        }
    };

    *state = config;

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

#[tauri::command]
fn command_handler(handle: tauri::AppHandle, state: State<'_, Mutex<Enigo>>, command: Command) {
    let mut enigo = state.lock().unwrap();
    handle_command(&handle, &mut *enigo, &command);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Enigo actions need to share the same enigo instance
    let enigo = Enigo::new(&Settings::default()).unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(enigo))
        .manage(Mutex::new(CurrentWindow::default()))
        .manage(Mutex::new(AppConfig::default()))
        .manage(Mutex::new(MacropadState {
            buttons: [ButtonState::None; 12],
            encoders: [0; 1],
        }))
        .setup(|app| {
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
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            command_handler
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn track_active_window(handle: &tauri::AppHandle) {
    let os = std::env::consts::OS;

    println!("OS: {}", os);

    loop {
        match match os {
            "windows" => get_current_window_windows(),
            _ => {
                println!("Unsupported OS");
                break;
            }
        } {
            Ok(current_window) => {
                let state_current_window = handle.state::<Mutex<CurrentWindow>>();
                let mut state_current_window = state_current_window.lock().unwrap();

                if state_current_window.title != current_window.title {
                    // Update the current window

                    println!("Current window: {}", current_window.title);
                    *state_current_window = current_window.clone();

                    // Emit an event to notify the frontend
                    handle
                        .emit("active-window-changed", current_window)
                        .unwrap();
                }
            }
            Err(e) => {
                println!("Failed to get current window: {}", e);
            }
        };

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn get_current_window_windows() -> Result<CurrentWindow> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            println!("Failed to get foreground window");
            anyhow::bail!("Failed to get foreground window");
        }

        // Get window title
        let mut title = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut title);
        let title = String::from_utf16_lossy(&title[..len as usize]);

        // Get process ID
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        // Open process
        let h_process = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
            Ok(h_process) => h_process,
            Err(e) => {
                println!("Failed to open process: {}", e);
                return Ok(CurrentWindow {
                    title,
                    app_name: "".to_string(),
                });
            }
        };
        if h_process.0 == std::ptr::null_mut() {
            return Ok(CurrentWindow {
                title,
                app_name: "".to_string(),
            });
        }

        // Get process name
        let mut exe_name = [0u16; 260];
        let len = K32GetModuleBaseNameW(h_process, None, &mut exe_name);
        let exe_name = String::from_utf16_lossy(&exe_name[..len as usize]);

        return Ok(CurrentWindow {
            title,
            app_name: exe_name,
        });
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

                    let mut buf: [u8; 2] = [0u8; 2]; // Buffer to hold the incoming data
                    loop {
                        match device.read(&mut buf[..]) {
                            Ok(0) => {
                                // No data read
                                // Sleep for a short duration to avoid busy-waiting
                                std::thread::sleep(std::time::Duration::from_millis(1));
                            }
                            Ok(n_bytes) => {
                                println!("Read: {:?}", &buf[..n_bytes]);

                                let application_profile = {
                                    let state_app_config = handle.state::<Mutex<AppConfig>>();
                                    let state_app_config = state_app_config.lock().unwrap();
                                    let state_current_window =
                                        handle.state::<Mutex<CurrentWindow>>();
                                    let state_current_window = state_current_window.lock().unwrap();

                                    // Iterate over all profiles and match regex against window title
                                    // First matching profile is taken
                                    state_app_config.application_profiles.iter().find_map(
                                        |(pattern, profile)| {
                                            if let Ok(re) = Regex::new(pattern) {
                                                if re.is_match(&state_current_window.title) {
                                                    Some(profile.clone())
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        },
                                    )
                                };

                                let macropad_state = handle.state::<Mutex<MacropadState>>();
                                let mut macropad_state = macropad_state.lock().unwrap();

                                let (new_macropad_state, action) =
                                    handle_report(macropad_state.clone(), buf);

                                let enigo = handle.state::<Mutex<Enigo>>();
                                let mut enigo = enigo.lock().unwrap();
                                perform_action(
                                    handle,
                                    &mut *enigo,
                                    &application_profile,
                                    macropad_state.clone(),
                                    action,
                                );

                                // Update the macropad state
                                *macropad_state = new_macropad_state;
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

fn perform_action(
    handle: &tauri::AppHandle,
    enigo: &mut Enigo,
    application_profile: &Option<ApplicationProfile>,
    _macropad_state: MacropadState,
    action: Action,
) {
    if application_profile.is_none() {
        return;
    }

    let profile = application_profile.as_ref().unwrap();

    // Handle actions not in profile
    match action {
        Action::ButtonRelease { id } => {
            // Retrieve inverse action
            if let Some(command) = profile.get_binding(&Action::ButtonPress { id }) {
                if let Some(_) = command.radial_menu_items {
                    handle.emit("hide-radial-menu", ()).unwrap();
                } else if let Some(operations) = command.operations {
                    let mut released_keys = HashSet::new();
                    for operation in operations.iter().rev() {
                        match operation {
                            Operation::KeyRelease { key } => {
                                released_keys.insert(key.clone());
                            }
                            Operation::KeyPress { key } => {
                                if released_keys.contains(key) {
                                    released_keys.remove(key);
                                    continue;
                                }
                                println!("Releasing key: {}", key);
                                enigo
                                    .key(key_to_enigo_key(&key), Direction::Release)
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                }
            }
            return;
        }
        _ => {}
    }

    if let Some(command) = profile.get_binding(&action) {
        handle_command(handle, enigo, &command);
    }
}

fn handle_command(handle: &tauri::AppHandle, enigo: &mut Enigo, command: &Command) {
    if let Some(radial_menu_items) = &command.radial_menu_items {
        show_radial_menu(handle, radial_menu_items);
    } else if let Some(operations) = &command.operations {
        for operation in operations {
            handle_operation(enigo, operation.clone());
        }
    }
}

fn show_radial_menu(handle: &tauri::AppHandle, items: &Vec<RadialMenuItem>) {
    let enigo = Enigo::new(&Settings::default()).unwrap();
    let mouse_location = enigo.location().unwrap();

    let event = events::ShowRadialMenu {
        location: mouse_location,
        items: items.iter().map(|item| (item).clone()).collect(),
    };

    println!("Emitting radial menu event: {:?}", event);
    handle.emit("show-radial-menu", event).unwrap();
}

fn handle_operation(enigo: &mut Enigo, operation: Operation) {
    match operation {
        Operation::KeyTap { key } => {
            println!("Tapping key: {}", key);
            enigo.key(key_to_enigo_key(&key), Direction::Click).unwrap();
        }
        Operation::KeyPress { key } => {
            println!("Pressing key: {}", key);
            enigo.key(key_to_enigo_key(&key), Direction::Press).unwrap();
        }
        Operation::KeyRelease { key } => {
            println!("Releasing key: {}", key);
            enigo
                .key(key_to_enigo_key(&key), Direction::Release)
                .unwrap();
        }
        Operation::Delay { ms } => {
            std::thread::sleep(std::time::Duration::from_millis(ms));
        }
        Operation::Repeat { times, operations } => {
            for _ in 0..times {
                for operation in operations.clone() {
                    handle_operation(enigo, operation);
                }
            }
        }
        _ => {
            println!("Unsupported operation: {operation:?}");
        }
    }
}

fn key_to_enigo_key(key: &str) -> Key {
    match key.to_uppercase().as_str() {
        "ESC" => return Key::Escape,
        "DEL" => return Key::Delete,
        "SHIFT" => return Key::Shift,
        "CTRL" => return Key::Control,
        "ALT" => return Key::Alt,
        "META" => return Key::Meta,
        key => Key::Unicode(key.to_lowercase().chars().next().unwrap()),
    }
}
