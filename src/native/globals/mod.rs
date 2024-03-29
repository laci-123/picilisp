use crate::memory::*;
use crate::debug::*;
use crate::error_utils::*;
use crate::util::*;
use super::NativeFunctionMetaData;



pub const WITH_CURRENT_MODULE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      with_current_module,
    name:          "with-current-module",
    kind:          FunctionKind::Lambda,
    parameters:    &["name", "module"],
    documentation: "Get value bound to `name`.
If it is defined in `module` then get it even if it is private."
};

pub fn with_current_module(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, WITH_CURRENT_MODULE.name, args, (let name: TypeLabel::Symbol), (let module: TypeLabel::Symbol)); 

    mem.get_global(&name.get_name(), &module.get_name()).map_err(|_| {
        let details = vec![("symbol", args[0].clone())];
        make_error(mem, "unbound-symbol", WITH_CURRENT_MODULE.name, &details)
    })
}


pub const FROM_MODULE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      from_module,
    name:          "from-module",
    kind:          FunctionKind::Lambda,
    parameters:    &["name", "module"],
    documentation: "Get value bound to `name` defined in `module`."
};

pub fn from_module(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, FROM_MODULE.name, args, (let name: TypeLabel::Symbol), (let module: TypeLabel::Symbol)); 

    match mem.get_global_from_module(&name.get_name(), &module.get_name()) {
        Ok(x) => Ok(x),
        Err(ModulError::GlobalNonExistentOrPrivate) => {
            let details = vec![("symbol", args[0].clone())];
            Err(make_error(mem, "unbound-symbol", FROM_MODULE.name, &details))
        },
        Err(ModulError::ModuleNonExistent) => {
            let details = vec![("module", args[1].clone())];
            Err(make_error(mem, "no-such-module", FROM_MODULE.name, &details))
        },
        _ => unreachable!(),
    }
}


pub const GET_CURRENT_MODULE: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      get_current_module,
    name:          "get-current-module",
    kind:          FunctionKind::Lambda,
    parameters:    &[],
    documentation: "Get current module."
};

pub fn get_current_module(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, GET_CURRENT_MODULE.name, args);    

    Ok(mem.symbol_for(&mem.get_current_module()))
}


pub const EXPORT: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      export,
    name:          "export",
    kind:          FunctionKind::Lambda,
    parameters:    &["names"],
    documentation: ""
};

pub fn export(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, EXPORT.name, args, (let names: TypeLabel::List));    

    for name in names {
        if let Some(PrimitiveValue::Symbol(s)) = name.get() {
            mem.add_export(&s.get_name());
        }
        else {
            let details = vec![("expected", mem.symbol_for("symbol-type")),
                               ("actual",   mem.symbol_for(name.get_type().to_string())),
                               ("symbol",   name)];
            return Err(make_error(mem, "wrong-argument-type", EXPORT.name, &details));
        }
    }

    Ok(mem.symbol_for("ok"))
}


pub const WHEREIS: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      whereis,
    name:          "whereis",
    kind:          FunctionKind::Lambda,
    parameters:    &["name"],
    documentation: "Return a list of modules where `name` is defined."
};

pub fn whereis(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, WHEREIS.name, args, (let name: TypeLabel::Symbol));    

    let vec = mem.get_module_of_global(&name.get_name()).iter().map(|x| mem.symbol_for(x)).collect::<Vec<GcRef>>();
    Ok(vec_to_list(mem, &vec))
}


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

    if let Some(meta) = args[0].get_meta() {
        let mut new_md       = meta.clone();
        new_md.documentation = documentation;
        let x = mem.allocate_metadata(value.clone_without_meta(), new_md);
        mem.define_global(&name.get_name(), x);
    }
    else {
        mem.define_global(&name.get_name(), value.clone());
    }

    if mem.is_global_exported(&name.get_name()) {
        let mut dm = DebugMessage::new();
        dm.insert("kind".to_string(), GLOBAL_DEFINED.to_string());
        dm.insert("name".to_string(), name.get_name());
        dm.insert("module".to_string(), mem.get_current_module());
        dm.insert("type".to_string(), value.get_type().to_string().to_string());
        match crate::native::print::print(mem, &[value], GcRef::nil(), recursion_depth + 1) {
            Ok(x)  => dm.insert("value".to_string(), list_to_string(x).unwrap()),
            Err(_) => dm.insert("value".to_string(), "#<ERROR: CANNOT CONVERT TO STRING>".to_string()),
        };
        if let Some(umb) = &mem.umbilical {
            umb.to_high_end.send(dm).expect("supervisor thread disappeared");
        }
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
    
