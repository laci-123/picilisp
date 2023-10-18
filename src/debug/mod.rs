use crate::memory::TypeLabel;
use std::{sync::mpsc::{Receiver, Sender, channel}, time::Instant};

pub enum DebugCommand {
    Abort,
    InterruptSignal,
    Pause,
    Resume,
    Step,
}


#[derive(Clone)]
pub enum StackFrame {
    Eval(String),
    Error(String),
}


#[derive(Clone)]
pub enum DiagnosticData {
    GlobalDefined {
        name: String,
        value: Result<String, String>,
        value_type: TypeLabel,
    },
    GlobalUndefined {
        name: String,
    },
    Memory {
        free_cells: usize,
        used_cells: usize,
        serial_number: usize,
    },
    CallStack {
        content: Vec<StackFrame>,
    },
    Paused,
}


pub struct UmbilicalHighEnd {
    pub to_low_end: Sender<DebugCommand>,
    pub from_low_end: Receiver<DiagnosticData>,
}


pub struct UmbilicalLowEnd {
    pub to_high_end: Sender<DiagnosticData>,
    pub from_high_end: Receiver<DebugCommand>,
    pub last_memory_send: Instant,
    pub serial_number: usize,
    pub paused: bool,
    pub in_step: bool,
    pub call_stack: Vec<StackFrame>,
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
        call_stack:       Vec::new(),
    };

    (high, low)
}
