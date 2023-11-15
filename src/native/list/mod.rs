use crate::memory::*;
use crate::util::*;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



pub const CONS: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      cons,
    name:          "cons",
    kind:          FunctionKind::Lambda,
    parameters:    &["car", "cdr"],
    documentation: "Create a new cons-cell with `car` and `cdr` as its two components.",
};

pub fn cons(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, CONS.name, args, (let car: TypeLabel::Any), (let cdr: TypeLabel::Any));

    Ok(mem.allocate_cons(car, cdr))
}


pub const CAR: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      car,
    name:          "car",
    kind:          FunctionKind::Lambda,
    parameters:    &["cons"],
    documentation: "Return the car of `cons`. Error if `cons` is not actually a cons-cell.",
};

pub fn car(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, CAR.name, args, (let cons: TypeLabel::Cons));

    Ok(cons.get_car())
}


pub const CDR: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      cdr,
    name:          "cdr",
    kind:          FunctionKind::Lambda,
    parameters:    &["cons"],
    documentation: "Return the cdr of `cons`. Error if `cons` is not actually a cons-cell.",
};

pub fn cdr(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, CDR.name, args, (let cons: TypeLabel::Cons));

    Ok(cons.get_cdr())
}


pub const LIST: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      list,
    name:          "list",
    kind:          FunctionKind::Lambda,
    parameters:    &["&", "objects"],
    documentation: "Return a newly created list with specified arguments as elements.
Allows any number of arguments, including zero."
};

pub fn list(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    // no need for validation: can accept any number of any type

    Ok(vec_to_list(mem, &args.to_vec()))
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


pub const GET_PROPERTY: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      get_property,
    name:          ".",
    kind:          FunctionKind::Lambda,
    parameters:    &["plist", "key"],
    documentation: "Return the value associated with `key` in `plist`.
Return nil of no value is associated with `key`.
Error if `plist` is not a valid property-list."
};

pub fn get_property(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, GET_PROPERTY.name, args, (let plist: TypeLabel::List), (let key: TypeLabel::Symbol));

    if let Some(result) = get_property_internal(key, &plist) {
        Ok(result)
    }
    else {
        Err(make_error(mem, "wrong-plist-format", GET_PROPERTY.name, &vec![]))
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

pub fn unrest(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, UNREST.name, args, (let f: TypeLabel::Function));

    if let Function::NormalFunction(nf) = f {
        let has_rest_params = false;
        let new_nf = mem.allocate_normal_function(nf.get_kind(), has_rest_params, nf.get_body(), &nf.get_params(), nf.get_env(), &nf.get_env_module());
        Ok(new_nf)
    }
    else {
        Ok(args[0].clone())
    }
}
