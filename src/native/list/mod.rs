use crate::memory::*;
use crate::util::*;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



pub fn cons(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "cons", &vec![ParameterType::Any, ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let cons = mem.allocate_cons(args[0].clone(), args[1].clone());

    NativeResult::Value(cons)
}


pub fn car(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "car", &vec![ParameterType::Type(TypeLabel::Cons)], args);
    if nr.is_err() {
        return nr;
    }

    NativeResult::Value(args[0].get().unwrap().as_conscell().get_car())
}


pub fn cdr(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "cdr", &vec![ParameterType::Type(TypeLabel::Cons)], args);
    if nr.is_err() {
        return nr;
    }

    NativeResult::Value(args[0].get().unwrap().as_conscell().get_cdr())
}


pub fn list(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    NativeResult::Value(vec_to_list(mem, &args.to_vec()))
}


fn get_property_internal(key: &Symbol, plist: &[GcRef]) -> Option<GcRef> {
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


pub fn property(mem: &mut Memory, key: &str, plist: GcRef) -> Option<GcRef> {
    list_to_vec(plist).and_then(|v| get_property_internal(&mem.symbol_for(key).get().unwrap().as_symbol(), &v))
}


pub fn get_property(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "get-property", &vec![ParameterType::Type(TypeLabel::Symbol), ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let key = args[0].get().unwrap().as_symbol();

    let plist;
    if let Some(v) = list_to_vec(args[1].clone()) {
        plist = v;
    }
    else {
        return NativeResult::Signal(mem.symbol_for("wrong-argument-type"));
    }

    if let Some(result) = get_property_internal(key, &plist) {
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


pub const UNREST: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      unrest,
    name:          "unrest",
    kind:          FunctionKind::Lambda,
    parameters:    &["f"],
    documentation: "Transform `f` so that its last parameter is a normal list and not a rest-parameter.
If `f` doesn't have rest-paramteres then it will remain unchanged.",
};

pub fn unrest(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, UNREST.name, &vec![ParameterType::Type(TypeLabel::Function)], args);
    if nr.is_err() {
        return nr;
    }

    if let Some(PrimitiveValue::Function(Function::NormalFunction(nf))) = args[0].get() {
        let has_rest_params = false;
        let new_nf = mem.allocate_normal_function(nf.get_kind(), has_rest_params, nf.get_body(), &nf.get_params(), nf.get_env());
        NativeResult::Value(new_nf)
    }
    else {
        NativeResult::Value(args[0].clone())
    }
}
