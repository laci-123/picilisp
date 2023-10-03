use crate::memory::*;
use crate::native::eval::eval;
use crate::util::list_to_string;
use super::NativeFunctionMetaData;



pub const DEFINE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      define,
    name:          "define",
    kind:          FunctionKind::SpecialLambda,
    parameters:    &["name", "value", "documentation"],
    documentation: "Define the symbol `name` as a global constant with `value` as its value
and the string `documentation` as the documentation field of its metadata.
Error if a global constant is already defined with the same name."
};

pub fn define(mem: &mut Memory, args: &[GcRef], env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    if args.len() != 3 {
        return Err(mem.symbol_for("wrong-arg-count"));
    }

    let name =
    if let Some(PrimitiveValue::Symbol(symbol)) = args[0].get() {
        symbol.get_name()
    }
    else {
        return Err(mem.symbol_for("definition-not-symbol"));
    };

    let value = eval(mem, &[args[1].clone()], env, recursion_depth + 1)?;

    let documentation =
    if !args[2].is_nil() {
        if let Some(string) = list_to_string(args[2].clone()) {
            string
        }
        else {
            return Err(mem.symbol_for("documentation-not-string"));
        }
    }
    else {
        return Err(mem.symbol_for("documentation-not-string"));
    };

    if mem.is_global_defined(&name) {
        return Err(mem.symbol_for("already-defined"));
    }

    if let Some(meta) = args[0].get_metadata() {
        let mut new_md         = meta.clone();
        new_md.documentation   = documentation;
        mem.define_global(&name, value.with_metadata(new_md));
    }
    else {
        mem.define_global(&name, value);
    }

    Ok(mem.symbol_for("ok"))
}


pub const UNDEFINE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      undefine,
    name:          "undefine",
    kind:          FunctionKind::SpecialLambda,
    parameters:    &["name"],
    documentation: "Delete the global constant associated with the symbol `name`, if any."
};

pub fn undefine(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    if args.len() != 1 {
        return Err(mem.symbol_for("wrong-arg-count"));
    }

    let name =
    if let Some(PrimitiveValue::Symbol(symbol)) = args[0].get() {
        symbol.get_name()
    }
    else {
        return Err(mem.symbol_for("definition-not-symbol"));
    };

    mem.undefine_global(&name);

    Ok(mem.symbol_for("ok"))
}
    
