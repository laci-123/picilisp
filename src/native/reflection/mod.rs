use crate::memory::*;
use crate::util::*;
use crate::native::list::make_plist;
use crate::error_utils::*;

pub fn type_of(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "type-of", &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let symbol = mem.symbol_for(args[0].get_type().to_string());

    NativeResult::Value(symbol)
}


pub fn get_metadata(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    let nr = validate_arguments(mem, "get-metadata", &vec![ParameterType::Any], args);
    if nr.is_err() {
        return nr;
    }

    let metadata    = args[0].get_metadata();
    let param_names = get_param_names(mem, args[0].clone());

    match (metadata, param_names.clone()) {
        (Some(md), _) => {
            let doc     = string_to_list(mem, &md.documentation);
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

            if let Some(pn) = param_names {
                vec.insert(0, ("parameters", pn));
            }

            if let Some(PrimitiveValue::Function(f)) = args[0].get() {
                let kind =
                match f.get_kind() {
                    FunctionKind::Macro         => mem.symbol_for("macro"),
                    FunctionKind::Syntax        => mem.symbol_for("syntax"),
                    FunctionKind::SpecialLambda => mem.symbol_for("special-lambda"),
                    FunctionKind::Lambda        => mem.symbol_for("lambda"),
                };
                vec.insert(0, ("function-kind", kind));
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

        return Some(vec_to_list(mem, &param_names));
    }
    else if let Some(md) = x.get_metadata() {
        if md.parameters.len() > 0 {
            let param_names = md.parameters.iter().map(|p| mem.symbol_for(p)).collect::<Vec<GcRef>>();
            return Some(vec_to_list(mem, &param_names));
        }
    }

    None
}
