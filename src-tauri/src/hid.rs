use crate::macropad_state::{ButtonState, MacropadState};
use crate::config::Action;

pub const VENDOR_ID: u16 = 0x1209;
pub const PRODUCT_ID: u16 = 0x001;
pub const USAGE_PAGE: u16 = 0xFF;
pub const USAGE: u16 = 0x01;

// Reports are sent on all change events
// Consequently there should only be one change event per report
// If somehow multiple change events are sent, only the last one will be processed
// All updates are persisted so that any inconsistencies can be corrected by the latest accurate report
pub fn handle_report(
    macropad_state: MacropadState,
    report: [u8; 2],
) -> (MacropadState, Action) {
    // First 12 bits of the report
    let buttons = ((report[1] as u16) << 8) | (report[0] as u16);
    // Next 2 bits as a 2 bit signed integer
    let mut encoders = vec![(report[1] >> 4) & 0b11].into_iter().map(|x| match x {
        0b00 => 0,
        0b01 => 1,
        0b11 => -1,
        _ => {
            eprintln!("Invalid encoder value: {}", x);
            0
        }
    });

    let mut new_macropad_state = macropad_state.clone();
    let mut action = Action::None;

    for i in 0..12 {
        let button_pressed = (buttons & (1 << i)) != 0;

        match (macropad_state.buttons[i], button_pressed) {
            (ButtonState::None, true) => {
                // Button was pressed
                println!("Button {} pressed", i);
                new_macropad_state.buttons[i] = ButtonState::Held {
                    pressed_at: std::time::Instant::now(),
                };
                action = Action::ButtonPress { button: i as u8 };
            }
            (ButtonState::Held { pressed_at: _ }, false) => {
                // Button was released
                println!("Button {} released", i);
                new_macropad_state.buttons[i] = ButtonState::None; // Reset to none state
                action = Action::ButtonRelease { button: i as u8 };
            }
            _ => {}
        }
    }

    for i in 0..encoders.clone().count() {
        let encoder_state = encoders.nth(i).unwrap();
        match (macropad_state.encoders[i], encoder_state) {
            (0, 1) => {
                println!("Encoder {} incremented", i);
                action = Action::EncoderIncrement { id: i as u8 };
            }
            (0, -1) => {
                println!("Encoder {} decremented", i);
                action = Action::EncoderDecrement { id: i as u8 };
            }
            _ => {}
        }
        new_macropad_state.encoders[i] = encoder_state;
    }

    return (new_macropad_state, action);
}