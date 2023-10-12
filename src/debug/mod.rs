use crate::{metadata::Metadata, memory::TypeLabel};
use std::sync::mpsc::{Receiver, Sender, channel};

pub enum DebugCommand {
    Abort,
    InterruptSignal,
}


pub enum DiagnosticData {
    GlobalDefined {
        name: String,
        value: Result<String, String>,
        value_type: TypeLabel,
        metadata: Option<Metadata>,
    },
    GlobalUndefined {
        name: String,
    }
}


pub struct UmbilicalHighEnd {
    pub to_low_end: Sender<DebugCommand>,
    pub from_low_end: Receiver<DiagnosticData>,
}


pub struct UmbilicalLowEnd {
    pub to_high_end: Sender<DiagnosticData>,
    pub from_high_end: Receiver<DebugCommand>,
}

impl UmbilicalLowEnd {
    pub fn init(&self) {
        // discard all messages that arrived earlier
        while let Ok(_) = self.from_high_end.try_recv() {}
    }
}


pub fn make_umbilical() -> (UmbilicalHighEnd, UmbilicalLowEnd) {
    let (high_to_low_tx, high_to_low_rx) = channel();
    let (low_to_high_tx, low_to_high_rx) = channel();
    (UmbilicalHighEnd{ to_low_end: high_to_low_tx,  from_low_end: low_to_high_rx },
     UmbilicalLowEnd{  to_high_end: low_to_high_tx, from_high_end: high_to_low_rx })
}
