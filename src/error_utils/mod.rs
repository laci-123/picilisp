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


pub fn extended_get_type(thing: GcRef) -> TypeLabel {
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


macro_rules! _cast {
    ($x:expr, $(TypeLabel::)?Nil) => {
        if $x.is_nil() {Some($x)} else {None}
    };
    ($x:expr, $(TypeLabel::)?Number) => {
        if let Some(PrimitiveValue::Number(y)) = $x.get() {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?Character) => {
        if let Some(PrimitiveValue::Character(y)) = $x.get() {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?Symbol) => {
        if let Some(PrimitiveValue::Symbol(y)) = $x.get() {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?Cons) => {
        if let Some(PrimitiveValue::Cons(y)) = $x.get() {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?String) => {
        if let Some(y) = list_to_string($x) {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?List) => {
        if let Some(y) = list_to_vec($x) {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?Function) => {
        if let Some(PrimitiveValue::Function(y)) = $x.get() {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?Trap) => {
        if let Some(PrimitiveValue::Trap(y)) = $x.get() {Some(y)} else {None}
    };
    ($x:expr, $(TypeLabel::)?Any) => {
        Some($x)
    };
}

pub(crate) use _cast as cast;


macro_rules! _count {
    ()                   => (0 as usize);
    ( $x:tt $($xs:tt)* ) => (1 + count!($($xs)*));
}

pub(crate) use _count as count;


macro_rules! _validate_args {
    ($mem:expr, $source:expr, $args:expr) => {
        let mem: &mut Memory = $mem;
        let source: &str = $source;
        let args: &[GcRef] = $args;

        if args.len() != 0 {
            let error_details = vec![("expected", mem.allocate_number(0)), ("actual", fit_to_number(mem, args.len()))];
            let error         = make_error(mem, "wrong-number-of-arguments", source, &error_details);
            return Err(error);
        }

    };
                                                             // Have to match a raw token-tree because cast! needs a literal, not an expression.
                                                             // Must be a sequence because e.g. TypeLabel::Number is a sequence of two tokens ("TypeLabel", "Number").
    ($mem:expr, $source:expr, $args:expr, $((let $name:ident : $($params:tt)+)),*) => {
        let mem: &mut Memory = $mem;
        let source: &str = $source;
        let args: &[GcRef] = $args;
        let params_count = count!($($name)*);

        if args.len() != params_count {
            let error_details = vec![("expected", fit_to_number(mem, params_count)), ("actual", fit_to_number(mem, args.len()))];
            let error         = make_error(mem, "wrong-number-of-arguments", source, &error_details);
            return Err(error);
        }

        let mut arg_iter = args.iter();
        $(
            let arg = arg_iter.next().unwrap().clone();
            let arg1 = arg.clone();
            let $name =
            {
                if let Some(x) = cast!(arg1, $($params)+) {
                    x
                }
                else {
                    let error_details = vec![("argument-value", arg.clone()),
                                             ("expected", mem.symbol_for($($params)+.to_string())),
                                             ("actual", mem.symbol_for(extended_get_type(arg.clone()).to_string()))];
                    let error         = make_error(mem, "wrong-argument-type", source, &error_details);
                    return Err(error);
                }
            };
        )*
    };
}

pub(crate) use _validate_args as validate_args;
