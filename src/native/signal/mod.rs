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

pub fn signal(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, SIGNAL.name, &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }
    
    NativeResult::Signal(args[0].clone())
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

pub fn trap(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, TRAP.name, &vec![ParameterType::Any, ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }
    
    let trap = mem.allocate_trap(args[0].clone(), args[1].clone());

    NativeResult::Value(trap)
}
