use crate::memory::*;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



pub const SIGNAL: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      signal,
    name:          "signal",
    kind:          FunctionKind::Lambda,
    parameters:    &["signal"],
    documentation: "Emit the signal `signal`."
};

pub fn signal(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, SIGNAL.name, args, (let signal: TypeLabel::Any));
    
    Err(signal)
}

