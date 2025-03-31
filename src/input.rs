use crate::cpu::is_bit_set;
use crate::memory_bus::JOYPAD_REGISTER;
use crate::{MEMORY_SIZE, MemoryBus, RustBoy};
use winit::keyboard::{KeyCode, PhysicalKey};

const SELECT_DIRECTION_BUTTON_BIT: u8 = 4;
const SELECT_ACTION_BUTTON_BIT: u8 = 5;

/// Struct to interact with the GameBoy joypad. The joypad state is represented by a single register
/// in the real GameBoy. The register has two flags which can be selected, depending on which
/// the rest of it represents the state of the action or direction buttons.
/// See: https://gbdev.io/pandocs/Joypad_Input.html
///
/// This struct is empty and has no fields and its purpose is just to provide static methods that
/// make it easier to interact with the joypad. The actual data is held at the [MemoryBus] struct.
///
/// We emulate the Joypad by having two separate states for the action and direction buttons, and always
/// writing to these whenever one of these buttons is pressed. The joypad_register keeps track of
/// which of the two states is currently selected. Using the [Joypad::get_joypad_register] and
/// [Joypad::write_joypad_register] methods, one can then read and write to the joypad register.
/// class then handles the logic of which button state is supposed to be returned.
pub struct Joypad {}

/// Struct to represent the state of the buttons on the joypad. Can either represent the action
/// or directional buttons. Not that, rather unconventionally, true indicates that a button is NOT
/// pressed and false indicates that a button IS pressed.
#[derive(Debug)]
pub(crate) struct ButtonState {
    start_or_down: bool,
    select_or_up: bool,
    b_or_left: bool,
    a_or_right: bool,
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
        Joypad::handle_button_press(&mut self.memory_bus, button);
    }

    /// Handles a button release event by calling the [Joypad::handle_button_release] method.
    pub fn handle_button_release(&mut self, button: Button) {
        Joypad::handle_button_release(&mut self.memory_bus, button);
    }
}

impl Joypad {
    /// Reads the joypad register and returns the value of the register.
    ///
    /// We set the upper two bits
    /// to 1 by default, whereas in the real RustBoy they have no purpose. Otherwise, this register
    /// 0xFF00 behaves as described in the [Pan Docs](https://gbdev.io/pandocs/Joypad_Input.html).
    pub fn get_joypad_register(memory_bus: &MemoryBus) -> u8 {
        let value: u8 = (memory_bus.memory[JOYPAD_REGISTER as usize] & 0b0011_0000) | 0b1100_0000;
        let select_action_button_flag = is_bit_set(value, SELECT_ACTION_BUTTON_BIT);
        let select_direction_button_flag = is_bit_set(value, SELECT_DIRECTION_BUTTON_BIT);
        match (!select_action_button_flag, !select_direction_button_flag) {
            (true, true) => {
                value
                    | (memory_bus.action_button_state.as_u8()
                        & memory_bus.direction_button_state.as_u8())
            }
            (true, false) => value | memory_bus.action_button_state.as_u8(),
            (false, true) => value | memory_bus.direction_button_state.as_u8(),
            (false, false) => value | 0x0F,
        }
    }

    /// Writes to the joypad register.
    ///
    /// Since bits 7,6 and the lower nibble are all not writable,
    /// only bits 5 and 4 of value will actually be considered.
    pub fn write_joypad_register(memory_bus: &mut MemoryBus, value: u8) {
        let value = value & 0b0011_0000;
        memory_bus.memory[JOYPAD_REGISTER as usize] = value;
    }

    /// Handles the button press event by setting the corresponding button state to false (pressed).
    pub(crate) fn handle_button_press(memory_bus: &mut MemoryBus, button: Button) {
        match button {
            Button::A => memory_bus.action_button_state.a_or_right = false,
            Button::B => memory_bus.action_button_state.b_or_left = false,
            Button::Start => memory_bus.action_button_state.start_or_down = false,
            Button::Select => memory_bus.action_button_state.select_or_up = false,
            Button::Up => memory_bus.direction_button_state.select_or_up = false,
            Button::Down => memory_bus.direction_button_state.start_or_down = false,
            Button::Left => memory_bus.direction_button_state.b_or_left = false,
            Button::Right => memory_bus.direction_button_state.a_or_right = false,
        }
        log::debug!("Button: {:?} pressed", button);
    }

    /// Handles the button release event by setting the corresponding button state to true (not pressed).
    pub(crate) fn handle_button_release(memory_bus: &mut MemoryBus, button: Button) {
        match button {
            Button::A => memory_bus.action_button_state.a_or_right = true,
            Button::B => memory_bus.action_button_state.b_or_left = true,
            Button::Start => memory_bus.action_button_state.start_or_down = true,
            Button::Select => memory_bus.action_button_state.select_or_up = true,
            Button::Up => memory_bus.direction_button_state.select_or_up = true,
            Button::Down => memory_bus.direction_button_state.start_or_down = true,
            Button::Left => memory_bus.direction_button_state.b_or_left = true,
            Button::Right => memory_bus.direction_button_state.a_or_right = true,
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
