use crate::interrupts::{Interrupt, InterruptFlagRegister};
use crate::{M_CYCLES_PER_SECOND, RustBoy};

const DIVIDER_REGISTER_FREQUENCY: u32 = 16_384;
const M_CYCLES_FOR_DIVIDER_REGISTER_INCREMENT: u32 =
    M_CYCLES_PER_SECOND / DIVIDER_REGISTER_FREQUENCY;
const DIVIDER_REGISTER_ADDRESS: usize = 0xFF04;
const TIMER_ADDRESS: u16 = 0xFF05;
const TIMER_MODULO_ADDRESS: u16 = 0xFF06;
const TIMER_CONTROL_ADDRESS: u16 = 0xFF07;

const TIMER_FREQUENCY_ZERO: u32 = 4_096;
const TIMER_FREQUENCY_ONE: u32 = 262_144;
const TIMER_FREQUENCY_TWO: u32 = 65_536;
const TIMER_FREQUENCY_THREE: u32 = 16_384;
const TIMER_FREQUENCY_ZERO_IN_M_CYCLES: u32 = M_CYCLES_PER_SECOND / TIMER_FREQUENCY_ZERO;
const TIMER_FREQUENCY_ONE_IN_M_CYCLES: u32 = M_CYCLES_PER_SECOND / TIMER_FREQUENCY_ONE;
const TIMER_FREQUENCY_TWO_IN_M_CYCLES: u32 = M_CYCLES_PER_SECOND / TIMER_FREQUENCY_TWO;
const TIMER_FREQUENCY_THREE_IN_M_CYCLES: u32 = M_CYCLES_PER_SECOND / TIMER_FREQUENCY_THREE;

pub struct TimerInfo {
    divider_running_m_cycle_counter: u32,
    timer_running_m_cycle_counter: u32,
}

impl TimerInfo {
    pub fn new() -> TimerInfo {
        TimerInfo {
            divider_running_m_cycle_counter: 0,
            timer_running_m_cycle_counter: 0,
        }
    }
}

impl RustBoy {
    /// Handles the timer and divider registers. This function is called every time the CPU makes
    /// a step, that is, executes an instruction, to check whether the timer and divider registers
    /// should be incremented. To do so, the functions [RustBoy::handle_divider] and
    /// [RustBoy::handle_timer] are called.
    pub fn handle_timer_and_divider(&mut self, cycles_passed: u32) {
        self.handle_divider(cycles_passed);
        self.handle_timer(cycles_passed);
    }

    /// Handles the incrementing of divider register. This register is incremented at a rate of
    /// [DIVIDER_REGISTER_FREQUENCY] Hz. This function is called every time the CPU makes
    /// a step, that is executes an instruction, to check whether the divider register should be
    /// incremented (converting Hz to CPU cycles, the divider register needs to be incremented every
    /// [M_CYCLES_FOR_DIVIDER_REGISTER_INCREMENT] cycles).
    fn handle_divider(&mut self, cycles_passed: u32) {
        self.timer_info.divider_running_m_cycle_counter += cycles_passed;
        if self.timer_info.divider_running_m_cycle_counter
            >= M_CYCLES_FOR_DIVIDER_REGISTER_INCREMENT
        {
            self.memory[DIVIDER_REGISTER_ADDRESS] =
                self.memory[DIVIDER_REGISTER_ADDRESS].wrapping_add(1);
            self.timer_info.divider_running_m_cycle_counter -=
                M_CYCLES_FOR_DIVIDER_REGISTER_INCREMENT;
        }
    }

    /// Handles the incrementing of the timer register at [TIMER_ADDRESS]. This register is
    /// incremented at the rate configured by the [TIMER_CONTROL_ADDRESS]. For more information, see
    /// https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#timer-and-divider-registers
    fn handle_timer(&mut self, cycles_passed: u32) {
        if self.is_timer_enabled() {
            self.timer_info.timer_running_m_cycle_counter += cycles_passed;
            let timer_frequency_in_m_cycles = self.get_timer_frequency_in_m_cycles();
            if self.timer_info.timer_running_m_cycle_counter >= timer_frequency_in_m_cycles {
                self.increment_timer();
                self.timer_info.timer_running_m_cycle_counter -= timer_frequency_in_m_cycles;
            }
        }
    }

    /// Increment the timer register and handle an overflow by setting the timer to the value
    /// provided in the [TIMER_MODULO_ADDRESS].
    fn increment_timer(&mut self) {
        let current_timer_value = self.read_byte(TIMER_ADDRESS);
        // Check if overflow is imminent
        if current_timer_value == 0xFF {
            // TODO: Possibly handle case, where TIMER MODULE REGISTER is edited in same m-cycle
            // as this happens and then old value is supposed to be used, see:
            // https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#ff06--tma-timer-modulo
            self.write_byte(TIMER_ADDRESS, self.get_timer_wraparound_value());
            // Request a timer interrupt
            InterruptFlagRegister::set_flag(&mut self.memory, Interrupt::Timer, true);
        } else {
            self.write_byte(TIMER_ADDRESS, current_timer_value.wrapping_add(1));
        }
    }

    /// Checks the timer control for whether the timer enabled bit is set and returns the result
    fn is_timer_enabled(&self) -> bool {
        self.read_byte(TIMER_CONTROL_ADDRESS) & 0b100 != 0
    }

    /// Checks the timer control for which timer frequency is selected and returns the frequency in
    /// #M-Cycles per Increment
    fn get_timer_frequency_in_m_cycles(&self) -> u32 {
        match self.read_byte(TIMER_CONTROL_ADDRESS) & 0b11 {
            0b00 => TIMER_FREQUENCY_ZERO_IN_M_CYCLES,
            0b01 => TIMER_FREQUENCY_ONE_IN_M_CYCLES,
            0b10 => TIMER_FREQUENCY_TWO_IN_M_CYCLES,
            0b11 => TIMER_FREQUENCY_THREE_IN_M_CYCLES,
            _ => unreachable!(),
        }
    }

    /// Checks the timer modulo address [TIMER_MODULO_ADDRESS] to determine the value the timer should reset to when it
    /// wraps around.
    fn get_timer_wraparound_value(&self) -> u8 {
        self.read_byte(TIMER_MODULO_ADDRESS)
    }
}
