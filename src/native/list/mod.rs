use crate::memory::*;
use crate::util::vec_to_list;



pub fn cons(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    let cons = mem.allocate_cons(args[0].clone(), args[1].clone());

    NativeResult::Value(cons)
}


pub fn car(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if let PrimitiveValue::Cons(cons) = args[0].get() {
        NativeResult::Value(cons.get_car())
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}


pub fn cdr(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    if let PrimitiveValue::Cons(cons) = args[0].get() {
        NativeResult::Value(cons.get_cdr())
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}


pub fn list(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    NativeResult::Value(vec_to_list(mem, args.to_vec()))
}
