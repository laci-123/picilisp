use crate::memory::*;
use crate::util::list_to_string;
use crate::error_utils::*;
use super::NativeFunctionMetaData;



pub const OUTPUT: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      output,
    name:          "output",
    kind:          FunctionKind::Lambda,
    parameters:    &["string"],
    documentation: "Print the string `string` to standard output.
Error if `string` is not a valid string."
};

pub fn output(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_arguments(mem, OUTPUT.name, &vec![ParameterType::Any], args)?;
    
    if let Some(msg) = list_to_string(args[0].clone()) {
        println!("{msg}");
    }
    else {
        let error = make_error(mem, "invalid-string", OUTPUT.name, &vec![]);
        return Err(error);
    }

    Ok(mem.symbol_for("ok"))
}
