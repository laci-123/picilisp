use crate::memory::*;
use crate::util::*;
use crate::native::list::make_plist;

pub fn type_of(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-number-of-arguments"));
    }

    if let Some(x) = args[0].get() {
        let symbol =
        match x {
            PrimitiveValue::Nil          => mem.symbol_for("nil"),
            PrimitiveValue::Number(_)    => mem.symbol_for("number"),
            PrimitiveValue::Character(_) => mem.symbol_for("character"),
            PrimitiveValue::Symbol(_)    => mem.symbol_for("symbol"),
            PrimitiveValue::Cons(_)      => mem.symbol_for("conscell"),
            PrimitiveValue::Trap(_)      => mem.symbol_for("trap"),
            PrimitiveValue::Function(f)  => {
                match f.get_kind() {
                    FunctionKind::Lambda        => mem.symbol_for("lambda"),
                    FunctionKind::SpecialLambda => mem.symbol_for("special-lambda"),
                    FunctionKind::Macro         => mem.symbol_for("macro"),
                    FunctionKind::Syntax        => mem.symbol_for("syntax-macro"),
                }
            },
            PrimitiveValue::Meta(_)      => unreachable!(),
        };

        NativeResult::Value(symbol)
    }
    else {
        NativeResult::Value(mem.symbol_for("nil"))
    }
}


pub fn get_metadata(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        return NativeResult::Signal(mem.symbol_for("wrong-number-of-arguments"));
    }

    let metadata    = args[0].get_metadata();
    let param_names = get_param_names(mem, args[0].clone());

    match (metadata, param_names.clone()) {
        (Some(md), _) => {
            let doc     = string_to_list(mem, &md.documentation);
            let file    = md.location.file.as_ref().map_or(mem.symbol_for("<stdin>"), |f| string_to_list(mem, &f.clone().into_os_string().into_string().unwrap()));
            let line    = mem.allocate_number(md.location.line as i64);
            let column  = mem.allocate_number(md.location.column as i64);
            let mut vec = vec![("documentation", doc), ("file", file), ("line", line), ("column", column)];

            if let Some(pn) = param_names {
                vec.insert(0, ("parameters", pn));
            }
            
            NativeResult::Value(make_plist(mem, &vec))
        },
        (None, Some(pn)) => {
            NativeResult::Value(make_plist(mem, &vec![("parameters", pn)]))
        },
        (None, None) => {
            NativeResult::Value(GcRef::nil())
        },
    }
}


fn get_param_names(mem: &mut Memory, x: GcRef) -> Option<GcRef> {
    if let Some(PrimitiveValue::Function(Function::NormalFunction(nf))) = x.get() {
        let mut param_names = nf.non_rest_params().collect::<Vec<GcRef>>();
        if let Some(rp) = nf.rest_param() {
            param_names.push(mem.symbol_for("&"));
            param_names.push(rp);
        }

        Some(vec_to_list(mem, &param_names))
    }
    else {
        None
    }
}
