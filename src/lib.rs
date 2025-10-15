#![no_std]

use embedded_hal::digital::InputPin;
use fugit::{Duration, ExtU64, Instant};

enum DebounceState {
    Off,
    DebouncingOff,
    DebouncingOn,
    On,
}

pub struct Button<P: InputPin> {
    pin: P,
    debounce_time: Duration<u64, 1, 1_000_000>,
    debounce_max: Duration<u64, 1, 1_000_000>,
    debounce_state: DebounceState,
    state: bool,
    active_state: bool,
    last_state: bool,
    toggle_state: bool,
    last_on: Instant<u64, 1, 1_000_000>,
    last_off: Instant<u64, 1, 1_000_000>,
}

impl<P: InputPin> Button<P> {
    pub fn new(pin: P, active_state: bool, debounce_time: Duration<u64, 1, 1_000_000>) -> Self {
        let debounce_max: Duration<u64, 1, 1_000_000> = 20_u64.millis();
        let zero_instant = Instant::<u64, 1, 1_000_000>::from_ticks(0);
        Button {
            pin: pin,
            debounce_time: debounce_time,
            debounce_max: debounce_max,
            debounce_state: DebounceState::Off,
            state: false,
            active_state: active_state,
            last_state: false,
            toggle_state: false,
            last_on: zero_instant,
            last_off: zero_instant,
        }
    }
    
    pub fn poll(&mut self, time: Instant<u64, 1, 1_000_000>) { 
        let current_state = match self.active_state {
            true => self.pin.is_high().unwrap(),
            false => self.pin.is_low().unwrap(),
        };
        let last_on_delta = time - self.last_on;
        let last_off_delta = time - self.last_off;
        self.last_state = self.state;
        
        match self.debounce_state {
            DebounceState::Off => {
                //~ rising edge, change state to DebouncingOn and record last_on time
                if current_state == true {
                    self.debounce_state = DebounceState::DebouncingOn;
                    self.last_on = time;
                }
            }
            DebounceState::DebouncingOff => {
                //~ pin state has stayed the same for the debounce_time, change debounce_state to Off
                if current_state == false && last_off_delta > self.debounce_time {
                    self.debounce_state = DebounceState::Off;
                    self.state = false;
                }
                //~ timeout, change state back to On
                if current_state == true && last_off_delta > self.debounce_max {
                    self.debounce_state = DebounceState::On;
                }
            }
            DebounceState::DebouncingOn => {
                //~ pin state has stayed the same for the debounce_time, change debounce_state to On
                if current_state == true && last_on_delta > self.debounce_time {
                    self.debounce_state = DebounceState::On;
                    self.state = true;
                    self.toggle_state = !self.toggle_state;
                }
                //~ timeout, change state back to Off
                if current_state == false && last_on_delta > self.debounce_max {
                    self.debounce_state = DebounceState::Off;
                }
            }
            DebounceState::On => {
                //~ falling edge, change state to DebouncingOff and record last_off time
                if current_state == false {
                    self.debounce_state = DebounceState::DebouncingOff;
                    self.last_off = time;
                }
            }
        }
    }
    
    pub fn state(&self) -> bool {
        self.state
    }
    
    pub fn last_state(&self) -> bool {
        self.last_state
    }
    
    pub fn toggle_state(&self) -> bool {
        self.toggle_state
    }
}
