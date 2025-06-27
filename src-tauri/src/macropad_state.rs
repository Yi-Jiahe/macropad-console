#[derive(Clone, Copy, Debug)]
pub enum ButtonState {
  None,
  Held{
    pressed_at: std::time::Instant,
  }
}

#[derive(Clone, Debug)]
pub struct MacropadState {
  pub buttons: [ButtonState; 12],
}