use crate::memory::*;
use crate::util::*;
use crate::native::read::read;
use crate::native::list::property;
use super::signal::get_signal_name;



fn lookup(mem: &mut Memory, key: GcRef, environment: GcRef) -> Option<GcRef> {
    let mut cursor = environment;

    while let Some(c) = cursor.get() {
        let cons = c.as_conscell();
        let key_value = cons.get_car();

        if key_value.get().unwrap().as_conscell().get_car().get().unwrap().as_symbol() == key.get().unwrap().as_symbol() {
            return Some(key_value.get().unwrap().as_conscell().get_cdr());
        }

        cursor = cons.get_cdr();
    }

    mem.get_global(&key.get().unwrap().as_symbol().get_name())
}


#[derive(PartialEq, Eq, Clone, Copy)]
enum Mode {
    Eval,
    MacroExpand,
}


struct AtomFrame {
    mode: Mode,
    value: GcRef,
    environment: GcRef,
    in_call: bool,
}


enum ConsProgress {
    NotStartedYet,
    EvalingCar,
    EvalingCdr,
}


struct ConsFrame {
    mode: Mode,
    car: GcRef,
    cdr: GcRef,
    progress: ConsProgress,
    environment: GcRef,
}


struct ListFrame {
    mode: Mode,
    eval_args: bool,
    elems: Vec<GcRef>,
    current: usize,
    environment: GcRef,
    in_call: bool,
}


enum StackFrame {
    Atom(AtomFrame),
    Cons(ConsFrame),
    List(ListFrame),
}

impl StackFrame {
    fn new(tree: GcRef, environment: GcRef, mode: Mode) -> Self {
        if let Some(vec) = list_to_vec(tree.clone()) {
            // tree is a list
            Self::List(ListFrame{ mode, eval_args: false, elems: vec, current: 0, environment, in_call: false })
        }
        else if let Some(PrimitiveValue::Cons(cons)) = tree.get() {
            // tree is a conscell but not a list
            // (a conscell is a list if its cdr is either a list or nil)
            Self::Cons(ConsFrame{ mode, car: cons.get_car(), cdr: cons.get_cdr(), progress: ConsProgress::NotStartedYet, environment })
        }
        else {
            // tree is an atom
            Self::Atom(AtomFrame{ mode, value: tree, environment, in_call: false })
        }
    }

    fn get_mode(&self) -> Mode {
        match self {
            Self::Atom(atom_frame) => atom_frame.mode,
            Self::Cons(cons_frame) => cons_frame.mode,
            Self::List(list_frame) => list_frame.mode,
        }
    }
}


enum EvalInternal {
    Return(GcRef),
    Call(GcRef, GcRef, Mode),
    TailCall(GcRef, GcRef, Mode),
    Signal(GcRef),
    Abort(String),
}

impl EvalInternal {
    fn from_nativeresult(nr: NativeResult) -> Self {
        match nr {
            NativeResult::Value(x)       => Self::Return(x),
            NativeResult::Signal(signal) => Self::Signal(signal),
            NativeResult::Abort(msg)     => Self::Abort(msg),
        }
    }
}


fn process_atom(mem: &mut Memory, frame: &mut AtomFrame, return_value: GcRef) -> EvalInternal {
    let atom = frame.value.clone();
    let env  = frame.environment.clone();

    // MACROEXPAND

    // if a macro is bound to atom then return that, otherwise just return the atom itself
    if frame.mode == Mode::MacroExpand {
        if let Some(PrimitiveValue::Symbol(_)) = atom.get() {
            if let Some(value) = lookup(mem, atom.clone(), env) {
                if let Some(PrimitiveValue::Function(f)) = value.get() {
                    if f.get_kind() == FunctionKind::Macro {
                        return EvalInternal::Return(value);
                    }
                }
            }
        }

        return EvalInternal::Return(atom);
    }

    // EVAL
    
    if frame.in_call {
        // The callee was the normal body of a trap.
        // It has already set the return value, here we just forward it.
        return EvalInternal::Return(return_value);
    }
    
    match atom.get().unwrap_or(&PrimitiveValue::Nil) {
        PrimitiveValue::Symbol(_) => {
            if let Some(value) = lookup(mem, atom, env) {
                EvalInternal::Return(value)
            }
            else {
                EvalInternal::Signal(mem.symbol_for("unbound-symbol"))
            }
        },
        PrimitiveValue::Trap(trap) => {
            frame.in_call = true;
            EvalInternal::Call(trap.get_normal_body(), env, frame.mode)
        },
        _ => {
            // numbers, characters and functions in non-call position don't get evaluated
            // and conscells are evaluated in process_cons
            EvalInternal::Return(atom)
        },
    }
}


fn process_cons(mem: &mut Memory, frame: &mut ConsFrame, return_value: GcRef) -> EvalInternal {
    match frame.progress {
        ConsProgress::NotStartedYet => {
            let car = frame.car.clone();
            let env = frame.environment.clone();
            frame.progress = ConsProgress::EvalingCar;
            EvalInternal::Call(car, env, frame.mode)
        },
        ConsProgress::EvalingCar => {
            frame.car = return_value.clone();

            let cdr = frame.cdr.clone();
            let env = frame.environment.clone();
            frame.progress = ConsProgress::EvalingCdr;
            EvalInternal::Call(cdr, env, frame.mode)
        },
        ConsProgress::EvalingCdr => {
            frame.cdr = return_value.clone();
            EvalInternal::Return(mem.allocate_cons(frame.car.clone(), frame.cdr.clone()))
        },
    }
}


fn process_list(mem: &mut Memory, frame: &mut ListFrame, return_value: GcRef) -> EvalInternal {
    // empty list evaluates to nil
    if frame.elems.len() == 0 {
        return EvalInternal::Return(GcRef::nil());
    }

    // receive the evaluated element and step to the next one
    if frame.in_call {
        frame.elems[frame.current] = return_value.clone();
        frame.current += 1;
        frame.in_call = false;
    }

    // the operator has just been evaluated, now we decide whether to evaluate the arguments
    if frame.current == 1 {
        match frame.mode {
            Mode::MacroExpand => {
                // when macroexpanding, expand every element of all lists regardless of what their first element is
                frame.eval_args = true;
            },
            Mode::Eval => {
                // when evaluating, only expand arguments of lambdas
                if let Some(PrimitiveValue::Function(f)) = frame.elems[0].get() {
                    frame.eval_args =  f.get_kind() == FunctionKind::Lambda;
                }
                else {
                    // first element of list is not a function.
                    // cannot evaluate non-functions
                    // (it also would be an error if we found an un-expanded macro when evaluating,
                    // but we assume this cannot happen because eval always calls macroexpand first)
                    return EvalInternal::Signal(mem.symbol_for("eval-bad-operator")); // (*)
                }
            },
        }
    }

    // evaluate the operator and maybe the operands
    let i = frame.current;
    if i < (if frame.eval_args {frame.elems.len()} else {1}) {
        let x         = frame.elems[i].clone();
        frame.in_call = true;
        let env       = frame.environment.clone();
        return EvalInternal::Call(x, env, frame.mode);
    }

    // when evaluating:     get operator
    // when macroexpanding: get operator or return whole list
    let operator;
    match frame.elems[0].get().unwrap_or(&PrimitiveValue::Nil) {
        // if f is macro     && mode is macroexpand => go ahead with f
        // if f is macro     && mode is evaluate    => return whole list
        // if f is not macro && mode is macroexpand => we assume this cannot happen because eval always calls macroexpand first
        // if f is not macro && mode is evaluate    => go ahead with f
        PrimitiveValue::Function(f) if (f.get_kind() == FunctionKind::Macro) == (frame.mode == Mode::MacroExpand) => {
            operator = f;
        },
        _ => {
            // when macroexpanding, if the first element is not a macro then return the whole list
            // (if evaluating and the first element is not a function, we already returned at (*))
            return EvalInternal::Return(vec_to_list(mem, &frame.elems));
        },
    }

    // call operator
    match operator {
        Function::NativeFunction(nf) => {
            EvalInternal::from_nativeresult(nf.call(mem, &frame.elems[1..], frame.environment.clone()))
        },
        Function::NormalFunction(nf) => {
            let new_tree = nf.get_body();
            let new_env  =
            match pair_params_and_args(mem, nf, &frame.elems[1..]) {
                EvalInternal::Return(ne) => ne,
                other                    => return other,
            };
            EvalInternal::TailCall(new_tree, new_env, Mode::Eval)
        },
    }
}


fn pair_params_and_args(mem: &mut Memory, nf: &NormalFunction, args: &[GcRef]) -> EvalInternal {
    let mut new_env = nf.get_env();

    let mut i = 0;
    for param in nf.non_rest_params() {
        let arg; 
        if let Some(a) = args.get(i) {
            arg = a.clone();
        }
        else {
            return EvalInternal::Signal(mem.symbol_for("not-enough-arguments"));
        };
        let param_arg = mem.allocate_cons(param, arg);
        new_env       = mem.allocate_cons(param_arg, new_env);

        i += 1;
    }

    if let Some(rest_param) = nf.rest_param() {
        let rest_args = vec_to_list(mem, &args[i..]);
        let param_arg = mem.allocate_cons(rest_param, rest_args);
        new_env       = mem.allocate_cons(param_arg, new_env);
    }
    else if i < args.len() {
        return EvalInternal::Signal(mem.symbol_for("too-many-arguments"));
    }

    EvalInternal::Return(new_env)
}


fn unwind_stack(mem: &mut Memory, stack: &mut Vec<StackFrame>, signal: GcRef) -> Option<StackFrame> {
    while let Some(old_frame) = stack.pop() {
        if let StackFrame::Atom(old_atom_frame) = old_frame {
            if let Some(PrimitiveValue::Trap(trap)) = old_atom_frame.value.get() {
                let trap_body          = trap.get_trap_body();
                let mut trap_env       = old_atom_frame.environment;
                let trapped_signal_sym = mem.symbol_for("*trapped-signal*");
                let key_value          = mem.allocate_cons(trapped_signal_sym, signal);
                trap_env               = mem.allocate_cons(key_value, trap_env);
                return Some(StackFrame::new(trap_body, trap_env, Mode::Eval));
            }
        }
    }

    None
}


fn process(mem: &mut Memory, tree: GcRef, env: GcRef, initial_mode: Mode) -> NativeResult {
    let mut stack        = vec![StackFrame::new(tree, env.clone(), initial_mode)];
    let mut return_value = GcRef::nil();

    while let Some(frame) = stack.last_mut() {
        let evaled =
        match frame {
            StackFrame::Atom(ref mut atom_frame) => {
                process_atom(mem, atom_frame, return_value.clone())
            },
            StackFrame::Cons(ref mut cons_frame) => {
                process_cons(mem, cons_frame, return_value.clone())
            },
            StackFrame::List(ref mut list_frame) => {
                process_list(mem, list_frame, return_value.clone())
            },
        };

        match evaled {
            EvalInternal::Return(x) => {
                return_value = x;
            },
            EvalInternal::Call(new_tree, new_env, mode) => {
                stack.push(StackFrame::new(new_tree, new_env, mode));
                continue;
            },
            EvalInternal::TailCall(new_tree, new_env, mode) => {
                *frame = StackFrame::new(new_tree, new_env, mode);
                continue;
            },
            EvalInternal::Signal(signal) => {
                match unwind_stack(mem, &mut stack, signal.clone()) {
                    Some(trap_frame) => {
                        stack.push(trap_frame);
                        continue;
                    }
                    None => {
                        return NativeResult::Signal(signal);
                    },
                }
            },
            EvalInternal::Abort(msg) => {
                return NativeResult::Abort(msg);
            },
        }

        let last_frame = stack.pop();

        if last_frame.is_some_and(|lf| lf.get_mode() == Mode::Eval) {
            if let Some(PrimitiveValue::Trap(_)) = return_value.get() {
                stack.push(StackFrame::new(return_value.clone(), env.clone(), Mode::Eval));
            }
        }
    }

    NativeResult::Value(return_value)
}


pub fn eval(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    if args.len() != 1 {
        let signal = mem.symbol_for("wrong-number-of-arguments");
        return NativeResult::Signal(signal);
    }

    let expanded =
    match process(mem, args[0].clone(), env.clone(), Mode::MacroExpand) {
        NativeResult::Value(e) => e,
        other                  => return other,
    };

    process(mem, expanded, env, Mode::Eval)
}


pub fn macroexpand(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    if args.len() != 1 {
        let signal = mem.symbol_for("wrong-number-of-arguments");
        return NativeResult::Signal(signal);
    }

    process(mem, args[0].clone(), env, Mode::MacroExpand)
}


pub fn eval_external(mem: &mut Memory, tree: GcRef) -> Result<GcRef, String> {
    let empty_env = GcRef::nil();
    
    match eval(mem, &[tree], empty_env.clone()) {
        NativeResult::Value(x)       => Ok(x),
        NativeResult::Signal(signal) => Err(format!("Unhandled signal: {}", get_signal_name(signal).unwrap_or("<unknown signal>".to_string()))),
        NativeResult::Abort(msg)     => Err(msg),
    }
}


pub fn load_all(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() != 1 {
        let signal = mem.symbol_for("wrong-number-of-arguments");
        return NativeResult::Signal(signal);
    }

    let ok_symbol         = mem.symbol_for("ok");
    let incomplete_symbol = mem.symbol_for("incomplete");
    let error_symbol      = mem.symbol_for("error");
    let invalid_symbol    = mem.symbol_for("invalid");

    let mut input  = args[0].clone();
    let mut line   = mem.allocate_number(1);
    let mut column = mem.allocate_number(1);

    while !input.is_nil() {
        let output     = match read(mem, &[input.clone(), line.clone(), column.clone()], GcRef::nil()) {
            NativeResult::Value(x) => x,
            other                  => return other,
        };
        let status = property(mem, "status", output.clone()).unwrap();
        let result = property(mem, "result", output.clone()).unwrap();
        let rest   = property(mem, "rest",   output.clone()).unwrap();
        line       = property(mem, "line",   output.clone()).unwrap();
        column     = property(mem, "column", output).unwrap();

        if symbol_eq!(status, ok_symbol) {
            match eval(mem, &[result], GcRef::nil()) {
                NativeResult::Value(_) => {/* only interested in side effects */},
                other                  => return other,
            }
        }
        else if symbol_eq!(status, incomplete_symbol) {
            return NativeResult::Signal(mem.symbol_for("input-incomplete"));
        }
        else if symbol_eq!(status, error_symbol) {
            return NativeResult::Signal(mem.symbol_for("read-error"));
        }
        else if symbol_eq!(status, invalid_symbol) {
            return NativeResult::Signal(mem.symbol_for("input-invalid-string"));
        }

        input = rest;
    }

    NativeResult::Value(ok_symbol)
}


#[cfg(test)]
mod tests;
