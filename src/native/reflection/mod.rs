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

    let metadata;
    if let Some(x) = args[0].get_metadata() {
        metadata = x;
    }
    else {
        return NativeResult::Value(GcRef::nil());
    }

    let doc    = string_to_list(mem, &metadata.documentation);
    let file   = metadata.location.file.as_ref().map_or(mem.symbol_for("<stdin>"), |f| string_to_list(mem, &f.clone().into_os_string().into_string().unwrap()));
    let line   = mem.allocate_number(metadata.location.line as i64);
    let column = mem.allocate_number(metadata.location.column as i64);
    let vec    = vec![("documentation", doc), ("file", file), ("line", line), ("column", column)];

    NativeResult::Value(make_plist(mem, &vec))
}
