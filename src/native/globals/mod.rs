use crate::memory::*;
use crate::debug::*;
use crate::error_utils::*;
use crate::util::*;
use crate::native::print::print_to_rust_string;
use super::NativeFunctionMetaData;



pub const DEFINE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      define,
    name:          "define",
    kind:          FunctionKind::Lambda,
    parameters:    &["name", "value", "documentation"],
    documentation: "Define the symbol `name` as a global constant with `value` as its value
and the string `documentation` as the documentation field of its metadata.
Error if a global constant is already defined with the same name."
};

pub fn define(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, DEFINE.name, args, (let name: TypeLabel::Symbol), (let value: TypeLabel::Any), (let documentation: TypeLabel::String));    

    if mem.is_global_defined(&name.get_name()) {
        return Err(mem.symbol_for("already-defined"));
    }

    if let Some(meta) = args[0].get_metadata() {
        let mut new_md         = meta.clone();
        new_md.documentation   = documentation;
        mem.define_global(&name.get_name(), value.clone().with_metadata(new_md));
    }
    else {
        mem.define_global(&name.get_name(), value.clone());
    }

    if let Some(umb) = &mem.umbilical {
        let mut dm = DebugMessage::new();
        dm.insert("kind".to_string(), GLOBAL_DEFINED.to_string());
        dm.insert("name".to_string(), name.get_name());
        dm.insert("type".to_string(), value.get_type().to_string().to_string());
        match print_to_rust_string(value, recursion_depth + 1) {
            Ok(x)  => dm.insert("value".to_string(), x),
            Err(_) => dm.insert("value".to_string(), "#<ERROR: CANNOT CONVERT TO STRING>".to_string()),
        };
        umb.to_high_end.send(dm).expect("supervisor thread disappeared");
    }

    Ok(mem.symbol_for("ok"))
}


pub const UNDEFINE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      undefine,
    name:          "undefine",
    kind:          FunctionKind::Lambda,
    parameters:    &["name"],
    documentation: "Delete the global constant associated with the symbol `name`, if any."
};

pub fn undefine(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, UNDEFINE.name, args, (let name: TypeLabel::Symbol));

    mem.undefine_global(&name.get_name());

    if let Some(umb) = &mem.umbilical {
        let mut dm = DebugMessage::new();
        dm.insert("kind".to_string(), GLOBAL_UNDEFINED.to_string());
        dm.insert("name".to_string(), name.get_name());
        umb.to_high_end.send(dm).expect("supervisor thread disappeared");
    }

    Ok(mem.symbol_for("ok"))
}
    
