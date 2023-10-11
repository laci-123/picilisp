use std::sync::mpsc::Receiver;

pub enum DebugCommand {
    Abort,
}


pub struct Umbilical {
    pub to_inferior: Receiver<DebugCommand>,
}

impl Umbilical {
    pub fn new(to_inferior: Receiver<DebugCommand>) -> Self {
        Self{ to_inferior }
    }
}
