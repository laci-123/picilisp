use std::sync::mpsc::{Receiver, Sender, channel};

pub enum DebugCommand {
    Abort,
    InterruptSignal,
}


pub struct UmbilicalHighEnd {
    pub to_low_end: Sender<DebugCommand>,
}


pub struct UmbilicalLowEnd {
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
    (UmbilicalHighEnd{ to_low_end: high_to_low_tx }, UmbilicalLowEnd{ from_high_end: high_to_low_rx })
}
