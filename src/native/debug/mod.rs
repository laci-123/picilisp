use crate::memory::*;
use crate::debug::*;
use crate::error_utils::*;
use crate::util::*;
use crate::native::print::print_to_rust_string;
use super::NativeFunctionMetaData;



pub const SEND: NativeFunctionMetaData =
NativeFunctionMetaData{
    function:      send,
    name:          "send",
    kind:          FunctionKind::Lambda,
    parameters:    &["data"],
    documentation: "Send `data` to the debugger.
`data` must be a valid property list.
If no debugger is attached, don't do anything."
};

pub fn send(mem: &mut Memory, args: &[GcRef], _env: GcRef, recursion_depth: usize) -> Result<GcRef, GcRef> {
    validate_args!(mem, SEND.name, args, (let data: TypeLabel::List));    

    let details = vec![("symbol", mem.symbol_for("data"))];
    let invalid_plist_error = make_error(mem, "invalid-plist", SEND.name, &details);
    if let Some(umb) = &mut mem.umbilical {
        let mut dm = DebugMessage::new();

        for d in data.chunks(2) {
            let key =
            if let Some(PrimitiveValue::Symbol(s)) = d[0].get() {
                s.get_name()
            }
            else {
                return Err(invalid_plist_error);
            };
            let value = print_to_rust_string(d[1].clone(), recursion_depth + 1).unwrap_or("#<ERROR: CANNOT CONVERT TO STRING>".to_string());
            dm.insert(key, value);
        }

        umb.to_high_end.send(dm).expect("supervisor thread disappeared");
    }

    Ok(mem.symbol_for("ok"))
}
