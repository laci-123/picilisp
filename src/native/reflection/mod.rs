use crate::memory::*;

pub fn type_of(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        let signal = mem.symbol_for("wrong-number-of-arguments");
        return NativeResult::Signal(signal);
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
