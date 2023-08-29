use crate::memory::*;
use crate::util::*;



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

    if let Some(PrimitiveValue::Cons(cons)) = args[0].get() {
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

    if let Some(PrimitiveValue::Cons(cons)) = args[0].get() {
        NativeResult::Value(cons.get_cdr())
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-arg-type"))
    }
}


pub fn list(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    NativeResult::Value(vec_to_list(mem, &args.to_vec()))
}


pub fn property(key: &Symbol, plist: &[GcRef]) -> Option<GcRef> {
    for x in plist.chunks(2) {
        if let Some(PrimitiveValue::Symbol(symbol)) = x[0].get() {
            if symbol == key {
                if let Some(v) = x.get(1) {
                    return Some(v.clone());
                }
                else {
                    return None;
                }
            }
        }
        else {
            return None;
        }
    }
    
    Some(GcRef::nil())
}


pub fn get_property(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 2 {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-count"));
    }

    let key;
    if let Some(PrimitiveValue::Symbol(symbol)) = args[0].get() {
        key = symbol;
    }
    else {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-type"));
    }

    let plist;
    if let Some(v) = list_to_vec(args[1].clone()) {
        plist = v;
    }
    else {
        return NativeResult::Signal(mem.symbol_for("wrong-arg-type"));
    }

    if let Some(result) = property(key, &plist) {
        NativeResult::Value(result)
    }
    else {
        NativeResult::Signal(mem.symbol_for("wrong-plist-format"))
    }
}


pub fn make_plist(mem: &mut Memory, kv: &[(&str, GcRef)]) -> GcRef {
    let mut vec = vec![];

    for (k, v) in kv {
        vec.push(mem.symbol_for(k));
        vec.push(v.clone());
    }

    vec_to_list(mem, &vec)
}
