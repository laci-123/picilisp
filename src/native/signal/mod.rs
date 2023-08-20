use crate::memory::*;



pub fn signal(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    
    NativeResult::Signal(args[0].clone())
}


pub fn trap(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }
    
    let trap = mem.allocate_trap(args[0].clone(), args[1].clone());

    NativeResult::Value(trap)
}
