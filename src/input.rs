use crate::RustBoy;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Struct to represent the joypad state. The joypad state is represented by a single register
/// in the real RustBoy. The register has two flags which can be selected, depending on which
/// the rest of it represents the state of the action or direction buttons.
/// See: https://gbdev.io/pandocs/Joypad_Input.html
///
/// We emulate this by having two separate states for the action and direction buttons, and always
/// writing to these whenever one of these buttons is pressed. The joypad_register keeps track of
/// which of the two states is currently selected. Using the [Joypad::read_joypad_register] and
/// [Joypad::write_joypad_register] methods, one can then read and write to the joypad. This class then
/// handles the logic of which button state is supposed to be returned.
pub struct Joypad {
    joypad_register: JoypadRegister,
    action_button_state: ButtonState,
    direction_button_state: ButtonState,
}

/// Struct to represent the state of the buttons on the joypad. Can either represent the action
/// or directional buttons. Not that, rather unconventionally, true indicates that a button is NOT
/// pressed and false indicates that a button IS pressed.
#[derive(Debug)]
struct ButtonState {
    start_or_down: bool,
    select_or_up: bool,
    b_or_left: bool,
    a_or_right: bool,
}

/// Struct to represent the joypad register. The register has two flags which can be selected,
/// depending on which the rest of it represents the state of the action or direction buttons.
/// The register in the real RustBoy contains more bits, but we outsourced these into the
/// ButtonState struct, see [Joypad]. Note that, rather unconventionally, true indicates that the respective
/// button type is NOT being read and false indicates that it IS being read. If both flags are
/// set to true, the joypad register will return 0xF for the lower nibble, indicating that no
/// buttons are pressed.
#[derive(Debug)]
struct JoypadRegister {
    select_action_buttons: bool,
    select_directional_buttons: bool,
}

/// Enum to represent the buttons on the joypad. The enum is used to identify which button is
/// pressed.
#[derive(Debug)]
pub enum Button {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

impl RustBoy {
    /// Handles a button press event by calling the [Joypad::handle_button_press] method.
    pub fn handle_button_press(&mut self, button: Button) {
        self.joypad.handle_button_press(button);
    }

    /// Handles a button release event by calling the [Joypad::handle_button_release] method.
    pub fn handle_button_release(&mut self, button: Button) {
        self.joypad.handle_button_release(button);
    }
}

impl Joypad {
    /// Reads the joypad register and returns the value of the register.
    ///
    /// We set the upper two bits
    /// to 1 by default, whereas in the real RustBoy they have no purpose. Otherwise, this register
    /// 0xFF00 behaves as described in the [Pan Docs](https://gbdev.io/pandocs/Joypad_Input.html).
    pub fn read_joypad_register(&self) -> u8 {
        let value: u8 = 0b1100_0000 | self.joypad_register.as_u8();
        match (
            !self.joypad_register.select_action_buttons,
            !self.joypad_register.select_directional_buttons,
        ) {
            (true, true) => {
                value | (self.action_button_state.as_u8() & self.direction_button_state.as_u8())
            }
            (true, false) => value | self.action_button_state.as_u8(),
            (false, true) => value | self.direction_button_state.as_u8(),
            (false, false) => value | 0x0F,
        }
    }

    /// Writes to the joypad register.
    ///
    /// Since bits 7,6 and the lower nibble are all not writable,
    /// only bits 5 and 4 of value will actually be considered.
    pub fn write_joypad_register(&mut self, value: u8) {
        self.joypad_register.select_action_buttons = (value & 0b0010_0000) != 0;
        self.joypad_register.select_directional_buttons = (value & 0b0001_0000) != 0;
    }

    /// Handles the button press event by setting the corresponding button state to false (pressed).
    pub(crate) fn handle_button_press(&mut self, button: Button) {
        match button {
            Button::A => self.action_button_state.a_or_right = false,
            Button::B => self.action_button_state.b_or_left = false,
            Button::Start => self.action_button_state.start_or_down = false,
            Button::Select => self.action_button_state.select_or_up = false,
            Button::Up => self.direction_button_state.select_or_up = false,
            Button::Down => self.direction_button_state.start_or_down = false,
            Button::Left => self.direction_button_state.b_or_left = false,
            Button::Right => self.direction_button_state.a_or_right = false,
        }
        log::debug!("Button: {:?} pressed", button);
    }

    /// Handles the button release event by setting the corresponding button state to true (not pressed).
    pub(crate) fn handle_button_release(&mut self, button: Button) {
        match button {
            Button::A => self.action_button_state.a_or_right = true,
            Button::B => self.action_button_state.b_or_left = true,
            Button::Start => self.action_button_state.start_or_down = true,
            Button::Select => self.action_button_state.select_or_up = true,
            Button::Up => self.direction_button_state.select_or_up = true,
            Button::Down => self.direction_button_state.start_or_down = true,
            Button::Left => self.direction_button_state.b_or_left = true,
            Button::Right => self.direction_button_state.a_or_right = true,
        }
    }

    /// Creates a new instance of the Joypad struct with all buttons set to not pressed and the
    /// joypad register set to none selected. This is the state of the joypad when no buttons are
    /// being pressed and when the RustBoy boots up.
    pub fn new_blank() -> Self {
        Joypad {
            joypad_register: JoypadRegister::new_nothing_selected(),
            action_button_state: ButtonState::new_nothing_pressed(),
            direction_button_state: ButtonState::new_nothing_pressed(),
        }
    }
}

impl ButtonState {
    /// Creates a new instance of the ButtonState struct with all buttons set to not pressed.
    /// Note that this means that the buttons' flags are all set to true.
    pub fn new_nothing_pressed() -> Self {
        ButtonState {
            start_or_down: true,
            select_or_up: true,
            b_or_left: true,
            a_or_right: true,
        }
    }

    fn as_u8(&self) -> u8 {
        let mut value: u8 = 0;
        if self.start_or_down {
            value |= 0b0000_1000;
        }
        if self.select_or_up {
            value |= 0b0000_0100;
        }
        if self.b_or_left {
            value |= 0b0000_0010;
        }
        if self.a_or_right {
            value |= 0b0000_0001;
        }
        value
    }
}

impl JoypadRegister {
    /// Creates a new instance of the JoypadRegister struct such that currently neither the action
    /// nor directional buttons are being read. Note that this means that both flags are set to
    /// true.
    pub fn new_nothing_selected() -> Self {
        JoypadRegister {
            select_action_buttons: true,
            select_directional_buttons: true,
        }
    }

    fn as_u8(&self) -> u8 {
        let mut value: u8 = 0;
        if self.select_action_buttons {
            value |= 0b0010_0000;
        }
        if self.select_directional_buttons {
            value |= 0b0001_0000;
        }
        value
    }
}

/// Handles the key pressed event by calling the [RustBoy::handle_button_press] method.
pub fn handle_key_pressed_event(rust_boy: &mut RustBoy, key: &PhysicalKey, paused: &mut bool) {
    match key {
        PhysicalKey::Code(KeyCode::ArrowLeft) => {
            rust_boy.handle_button_press(Button::Left);
        }
        PhysicalKey::Code(KeyCode::ArrowRight) => {
            rust_boy.handle_button_press(Button::Right);
        }
        PhysicalKey::Code(KeyCode::ArrowUp) => {
            rust_boy.handle_button_press(Button::Up);
        }
        PhysicalKey::Code(KeyCode::ArrowDown) => {
            rust_boy.handle_button_press(Button::Down);
        }
        PhysicalKey::Code(KeyCode::KeyA) => {
            rust_boy.handle_button_press(Button::A);
        }
        PhysicalKey::Code(KeyCode::KeyB) => {
            rust_boy.handle_button_press(Button::B);
        }
        PhysicalKey::Code(KeyCode::Enter) => {
            rust_boy.handle_button_press(Button::Start);
        }
        PhysicalKey::Code(KeyCode::Space) => {
            rust_boy.handle_button_press(Button::Select);
        }
        PhysicalKey::Code(KeyCode::KeyP) => {
            *paused = !*paused;
            if *paused {
                log::info!("Paused");
            } else {
                log::info!("Unpaused");
            }
        }
        _ => {}
    }
}

/// Handles the key released event by calling the [RustBoy::handle_button_release] method.
pub fn handle_key_released_event(rust_boy: &mut RustBoy, key: &PhysicalKey) {
    match key {
        PhysicalKey::Code(KeyCode::ArrowLeft) => {
            rust_boy.handle_button_release(Button::Left);
        }
        PhysicalKey::Code(KeyCode::ArrowRight) => {
            rust_boy.handle_button_release(Button::Right);
        }
        PhysicalKey::Code(KeyCode::ArrowUp) => {
            rust_boy.handle_button_release(Button::Up);
        }
        PhysicalKey::Code(KeyCode::ArrowDown) => {
            rust_boy.handle_button_release(Button::Down);
        }
        PhysicalKey::Code(KeyCode::KeyA) => {
            rust_boy.handle_button_release(Button::A);
        }
        PhysicalKey::Code(KeyCode::KeyB) => {
            rust_boy.handle_button_release(Button::B);
        }
        PhysicalKey::Code(KeyCode::Enter) => {
            rust_boy.handle_button_release(Button::Start);
        }
        PhysicalKey::Code(KeyCode::Space) => {
            rust_boy.handle_button_release(Button::Select);
        }
        _ => {}
    }
}
