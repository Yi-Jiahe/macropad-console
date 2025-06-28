#[derive(Clone, Copy, Debug)]
pub enum ButtonState {
  None,
  Held{
    pressed_at: std::time::Instant,
  }
}

#[derive(Clone, Copy, Debug)]
pub struct MacropadState {
  pub buttons: [ButtonState; 12],
  pub encoders: [i8; 1],
}