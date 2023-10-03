use crate::memory::*;
use crate::native::list::make_plist;
use crate::util::*;



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


fn extended_get_type(thing: GcRef) -> TypeLabel {
    match thing.get_type() {
        TypeLabel::Cons => {
            let ct = cons_type(thing);
            if ct.is_string {
                TypeLabel::String
            }
            else if ct.is_list {
                TypeLabel::List
            }
            else {
                TypeLabel::Cons
            }
        },
        other => other,
    }
}


pub enum ParameterType {
    Any,
    Type(TypeLabel),
}


macro_rules! cast {
    ($x:expr, $type:literal) => {
        match $type {
            TypeLabel::Nil       => if $x.is_nil()                                                {Some(x)} else {None},
            TypeLabel::Number    => if let Some(PrimitiveValue::Number(y))    = x                 {Some(y)} else {None},
            TypeLabel::Character => if let Some(PrimitiveValue::Character(y)) = x                 {Some(y)} else {None},
            TypeLabel::Cons      => if let Some(PrimitiveValue::Cons(y))      = x                 {Some(y)} else {None},
            TypeLabel::List      => if let Some(y)                            = list_to_vec(x)    {Some(y)} else {None},
            TypeLabel::String    => if let Some(y)                            = list_to_string(x) {Some(y)} else {None},
            TypeLabel::Symbol    => if let Some(PrimitiveValue::Symbol(y))    = x                 {Some(y)} else {None},
            TypeLabel::Function  => if let Some(PrimitiveValue::Function(y))  = x                 {Some(y)} else {None},
            TypeLabel::Trap      => if let Some(PrimitiveValue::Tryp(y))      = x                 {Some(y)} else {None},
        }
    };
}


macro_rules! nth_arg {
    ($mem:expr, $source:expr, $args:expr, $n:expr, $type:expr) => {
        let arg = $args[$n].clone();
        if let Some(x) = cast!(arg, $type) {
            Ok(x)
        }
        else {
            let error_details = vec![("expected", mem.symbol_for($type.to_string())), ("actual", mem.symbol_for(extended_get_type(arg.clone()).to_string()))];
            let error         = make_error(mem, "wrong-argument-type", $source, &error_details);
            Err(error)
        }
    };
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
