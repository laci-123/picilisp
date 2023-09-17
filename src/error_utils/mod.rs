use crate::memory::*;
use crate::native::list::make_plist;



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


pub enum ParameterType {
    Any,
    Type(TypeLabel),
}


pub fn validate_arguments(mem: &mut Memory, source: &str, parameters: &[ParameterType], arguments: &[GcRef]) -> Result<GcRef, GcRef> {
    if parameters.len() != arguments.len() {
        let error_details = vec![("expected", fit_to_number(mem, parameters.len())), ("actual", fit_to_number(mem, arguments.len()))];
        let error         = make_error(mem, "wrong-number-of-arguments", source, &error_details);
        return Err(error);
    }

    for (p, arg) in parameters.iter().zip(arguments) {
        let a_type = arg.get_type();
        if let ParameterType::Type(p_type) = p {
            if a_type != *p_type {
                let error_details = vec![("expected", mem.symbol_for(p_type.to_string())), ("actual", mem.symbol_for(a_type.to_string()))];
                let error         = make_error(mem, "wrong-argument-type", source, &error_details);
                return Err(error);
            }
        }
    }

    Ok(GcRef::nil())
}
