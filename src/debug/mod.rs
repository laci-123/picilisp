use std::{sync::mpsc::{Receiver, Sender, channel}, time::Instant, collections::HashMap};



pub const GLOBAL_DEFINED: &str = "GLOBAL_DEFINED";
pub const GLOBAL_UNDEFINED: &str = "GLOBAL_UNDEFINED";


pub type DebugMessage = HashMap<String, String>;


pub struct UmbilicalHighEnd {
    pub to_low_end: Sender<DebugMessage>,
    pub from_low_end: Receiver<DebugMessage>,
}


pub struct UmbilicalLowEnd {
    pub to_high_end: Sender<DebugMessage>,
    pub from_high_end: Receiver<DebugMessage>,
    pub last_memory_send: Instant,
    pub serial_number: usize,
    pub paused: bool,
    pub in_step: bool,
}


pub fn make_umbilical() -> (UmbilicalHighEnd, UmbilicalLowEnd) {
    let (high_to_low_tx, high_to_low_rx) = channel();
    let (low_to_high_tx, low_to_high_rx) = channel();
    let high = UmbilicalHighEnd {
        to_low_end: high_to_low_tx,
        from_low_end: low_to_high_rx,
    };
    let low  = UmbilicalLowEnd {
        to_high_end:      low_to_high_tx,
        from_high_end:    high_to_low_rx,
        last_memory_send: Instant::now(),
        serial_number:    0,
        paused:           false,
        in_step:          false,
    };

    (high, low)
}
