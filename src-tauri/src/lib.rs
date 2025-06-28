use std::sync::Mutex;

use anyhow::Result;
use dirs::home_dir;
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
use crate::config::{Action, AppConfig, ApplicationAction, ApplicationProfile};
use crate::hid::{PRODUCT_ID, USAGE, USAGE_PAGE, VENDOR_ID};
use crate::macropad_state::{ButtonState, MacropadState};

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

#[tauri::command]
fn handle_action(action: ApplicationAction) {
    match action {
        ApplicationAction::KeyTap { .. } => {
            handle_key_action(action);
        }
        ApplicationAction::MacroTap { actions } => {
            handle_macro_tap(&actions);
        }
        _ => {
            println!("Unsupported action: {action:?}");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(CurrentWindow::default()));
            app.manage(Mutex::new(AppConfig::default()));
            app.manage(Mutex::new(MacropadState {
                buttons: [ButtonState::None; 12],
            }));

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
            handle_action
        ])
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
    let os = std::env::consts::OS;

    println!("OS: {}", os);

    loop {
        match (match os {
            "windows" => get_current_window_windows(),
            _ => {
                println!("Unsupported OS");
                break;
            }
        }) {
            Ok(current_window) => {
                // Update the current window
                let state_current_window = handle.state::<Mutex<CurrentWindow>>();
                let mut state_current_window = state_current_window.lock().unwrap();
                *state_current_window = current_window.clone();

                // Emit an event to notify the frontend
                handle
                    .emit("active-window-changed", current_window)
                    .unwrap();
            }
            Err(e) => {
                println!("Failed to get current window: {}", e);
            }
        };

        std::thread::sleep(std::time::Duration::from_secs(1));
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
        println!("Title: {}", title);

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

                                let state_macropad_state = handle.state::<Mutex<MacropadState>>();
                                let mut state_macropad_state = state_macropad_state.lock().unwrap();

                                let new_macropad_state = handle_report(
                                    handle,
                                    &application_profile,
                                    state_macropad_state.clone(),
                                    buf,
                                );

                                // Update the macropad state
                                *state_macropad_state = new_macropad_state;
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
    handle: &tauri::AppHandle,
    application_profile: &Option<ApplicationProfile>,
    macropad_state: MacropadState,
    report: [u8; 2],
) -> MacropadState {
    let buttons = ((report[1] as u16) << 8) | (report[0] as u16);
    let mut new_macropad_state = macropad_state.clone();

    for i in 0..12 {
        let button_pressed = (buttons & (1 << i)) != 0;

        match (macropad_state.buttons[i], button_pressed) {
            (ButtonState::None, true) => {
                println!("Button {} pressed", i);
                perform_action(
                    handle,
                    application_profile,
                    Action::ButtonPress { button: i as u8 },
                );
                // Button was pressed
                new_macropad_state.buttons[i] = ButtonState::Held {
                    pressed_at: std::time::Instant::now(),
                };
            }
            (ButtonState::Held { pressed_at: _ }, false) => {
                println!("Button {} released", i);
                perform_action(
                    handle,
                    application_profile,
                    Action::ButtonRelease { button: i as u8 },
                );
                // Button was released
                new_macropad_state.buttons[i] = ButtonState::None; // Reset to none state
            }
            _ => {}
        }
    }

    return new_macropad_state;
}

fn perform_action(
    handle: &tauri::AppHandle,
    application_profile: &Option<ApplicationProfile>,
    action: Action,
) {
    if application_profile.is_none() {
        return;
    }

    let profile = application_profile.as_ref().unwrap();

    // Handle actions not in profile
    match action {
        Action::ButtonRelease { button } => {
            let config_action = Action::ButtonPress { button };
            if let Some((_, application_action)) =
                profile.actions.iter().find(|(a, _)| *a == config_action)
            {
                match application_action {
                    ApplicationAction::KeyPress { key } => {
                        let complement_action = ApplicationAction::KeyRelease {
                            key: key.to_string(),
                        };
                        handle_key_action(complement_action);
                    }
                    ApplicationAction::OpenRadialMenu { .. } => {
                        handle.emit("hide-radial-menu", ()).unwrap();
                    }
                    _ => {}
                }
            }
            return;
        }
        _ => {}
    }

    // Find action in profile
    if let Some((_, application_action)) = profile.actions.iter().find(|(a, _)| *a == action) {
        match application_action {
            ApplicationAction::OpenRadialMenu { items } => {
                let enigo = Enigo::new(&Settings::default()).unwrap();
                let mouse_location = enigo.location().unwrap();

                let event = events::ShowRadialMenu {
                    location: mouse_location,
                    items: items.iter().map(|item| (**item).clone()).collect(),
                };

                println!("Emitting radial menu event: {:?}", event);
                handle.emit("show-radial-menu", event).unwrap();
            }
            ApplicationAction::KeyPress { .. } | ApplicationAction::KeyTap { .. } => {
                handle_key_action(application_action.clone());
            }
            ApplicationAction::MacroTap { actions } => {
                handle_macro_tap(actions);
            }
            _ => {}
        }
    }
}

fn handle_key_action(action: ApplicationAction) {
    if !matches!(
        action,
        ApplicationAction::KeyPress { .. }
            | ApplicationAction::KeyTap { .. }
            | ApplicationAction::KeyRelease { .. }
    ) {
        println!("Unsupported action: {action:?}");
        return;
    };

    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    match action {
        ApplicationAction::KeyTap { key } => {
            println!("Tapping key: {}", key);
            enigo.key(key_to_enigo_key(&key), Direction::Click).unwrap();
        }
        ApplicationAction::KeyPress { key } => {
            println!("Pressing key: {}", key);
            enigo.key(key_to_enigo_key(&key), Direction::Press).unwrap();
        }
        ApplicationAction::KeyRelease { key } => {
            println!("Releasing key: {}", key);
            enigo
                .key(key_to_enigo_key(&key), Direction::Release)
                .unwrap();
        }
        _ => unreachable!(),
    }
}

fn handle_macro_tap(actions: &Vec<ApplicationAction>) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    for action in actions.iter() {
        match action {
            ApplicationAction::KeyPress { key } => {
                println!("Pressing key: {}", key);
                enigo.key(key_to_enigo_key(&key), Direction::Press).unwrap();
            }
            ApplicationAction::KeyRelease { key } => {
                println!("Releasing key: {}", key);
                enigo
                    .key(key_to_enigo_key(&key), Direction::Release)
                    .unwrap();
            }
            ApplicationAction::KeyTap { key } => {
                println!("Tapping key: {}", key);
                enigo.key(key_to_enigo_key(&key), Direction::Click).unwrap();
            }
            _ => {
                println!("Unsupported action: {action:?}");
            }
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
