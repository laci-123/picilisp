use crate::memory::*;
use crate::metadata::*;
use crate::util::*;
use crate::native::list::make_plist;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



pub const DESTRUCTURE_TRAP: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      destructure_trap,
    name:          "destructure-trap",
    kind:          FunctionKind::Lambda,
    parameters:    &["trap"],
    documentation: "Return a list containing the normal-body and trap-body of `trap`."
};

pub fn destructure_trap(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, DESTRUCTURE_TRAP.name, args, (let t: TypeLabel::Trap));

    let vec = vec![t.get_normal_body(), t.get_trap_body()];
    Ok(vec_to_list(mem, &vec))
}


pub const DESTRUCTURE_FUNCTION: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      destructure_function,
    name:          "destructure-function",
    kind:          FunctionKind::Lambda,
    parameters:    &["function"],
    documentation: "Return a property list containing
the kind, the parameter list, body, captured environment and captured module of `function`.
For native functions and special operators body will be nil."
};

pub fn destructure_function(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, DESTRUCTURE_FUNCTION.name, args, (let f: TypeLabel::Function));

    let params = f.get_param_names().iter().map(|pn| mem.symbol_for(&pn)).collect::<Vec<GcRef>>();
    let vec    = vec![mem.symbol_for("kind"),        mem.symbol_for(f.get_kind().to_string()),
                      mem.symbol_for("parameters"),  vec_to_list(mem, &params),
                      mem.symbol_for("body"),        f.get_body(),
                      mem.symbol_for("environment"), f.get_env(),
                      mem.symbol_for("module"),      mem.symbol_for(&f.get_module())];
    Ok(vec_to_list(mem, &vec))
}


pub const TYPE_OF: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      type_of,
    name:          "type-of",
    kind:          FunctionKind::Lambda,
    parameters:    &["object"],
    documentation: "Return a symbol representing the type of `object`."
};

pub fn type_of(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, TYPE_OF.name, args, (let x: TypeLabel::Any));

    match x.get_type() {
        TypeLabel::Cons => {
            let ct = cons_type(args[0].clone());
            if ct.is_string {
                Ok(mem.symbol_for("string-type"))
            }
            else if ct.is_list {
                Ok(mem.symbol_for("list-type"))
            }
            else {
                Ok(mem.symbol_for("cons-type"))
            }
        },
        _ => {
            Ok(mem.symbol_for(x.get_type().to_string()))
        },
    }
}


pub const GET_METADATA: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      get_metadata,
    name:          "get-metadata",
    kind:          FunctionKind::Lambda,
    parameters:    &["object"],
    documentation: "Return all metadata stored about `object` in a property-list."
};

pub fn get_metadata(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, GET_METADATA.name, args, (let x: TypeLabel::Any));

    let metadata = x.get_meta();

    match metadata {
        Some(md) => {
            let doc = string_to_list(mem, &md.documentation);
            let file;
            let line;
            let column;
            match &md.location {
                Location::Native                             => {
                    file = mem.symbol_for("native");
                    line = GcRef::nil();
                    column = GcRef::nil();
                },
                Location::Prelude{ line: ln, column: cn }    => {
                    file = mem.symbol_for("prelude");
                    line = mem.allocate_number(*ln as i64);
                    column = mem.allocate_number(*cn as i64);
                },
                Location::Stdin{ line: ln, column: cn }      => {
                    file = mem.symbol_for("stdin");
                    line = mem.allocate_number(*ln as i64);
                    column = mem.allocate_number(*cn as i64);
                },
                Location::File{ path, line: ln, column: cn } => {
                    file = string_to_proper_list(mem, &path.clone().into_os_string().into_string().unwrap());
                    line = mem.allocate_number(*ln as i64);
                    column = mem.allocate_number(*cn as i64);
                },
            }
            let mut vec = vec![("documentation", doc), ("file", file)];

            if !line.is_nil() {
                vec.push(("line", line));
            }

            if !column.is_nil() {
                vec.push(("column", column));
            }
 
            Ok(make_plist(mem, &vec))
        },
        None => {
            Ok(GcRef::nil())
        },
    }
}

