use crate::{memory::*, native::list::make_plist};

use super::list::property;



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


pub fn make_error(mem: &mut Memory, kind: &str, source: &str, details: &[(&str, GcRef)]) -> GcRef {
    let mut vec = vec![("kind", mem.symbol_for(kind)), ("source", mem.symbol_for(source))];
    vec.extend_from_slice(details);
    make_plist(mem, &vec)
}


pub fn fit_to_number(mem: &mut Memory, x: usize) -> GcRef {
    if let Ok(y) = i64::try_from(x) {
        mem.allocate_number(y)
    }
    else {
        mem.symbol_for("more-than-number-type-maximum")
    }
}


pub fn get_error_kind(mem: &mut Memory, error: GcRef) -> Option<String> {
    if let Some(x) = property(mem, "kind", error) {
        if let Some(PrimitiveValue::Symbol(symbol)) = x.get() {
            Some(symbol.get_name())
        }
        else {
            None
        }
    }
    else {
        None
    }
}
