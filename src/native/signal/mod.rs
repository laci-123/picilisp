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


pub const TRAP: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      trap,
    name:          "trap",
    kind:          FunctionKind::SpecialLambda,
    parameters:    &["normal-body", "trap-body"],
    documentation: "Create a trap with `normal-body` and `trap-body`.
When a trap is evaluated,
first the normal body is evaluated.
If during the evaluation of the normal body a signal is emitted,
the evaluation is tranfered to the trap body
where the symbol *trapped-signal* is bound to the emitted signal.
If no signal is emitted during the evaluation of the normal body
then the trap body is never evaluated."
};

pub fn trap(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, TRAP.name, args, (let normal_body: TypeLabel::Any), (let trap_body: TypeLabel::Any));
    
    Ok(mem.allocate_trap(normal_body, trap_body))
}
