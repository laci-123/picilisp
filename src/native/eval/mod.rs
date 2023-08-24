use crate::memory::*;
use crate::util::*;
use crate::native::print::print;



fn lookup(mem: &Memory, key: GcRef, environment: GcRef) -> Option<GcRef> {
    let mut cursor = environment;

    while !cursor.is_nil() {
        let cons = cursor.get().as_conscell();
        let key_value = cons.get_car();

        if key_value.get().as_conscell().get_car().get().as_symbol() == key.get().as_symbol() {
            return Some(key_value.get().as_conscell().get_cdr());
        }

        cursor = cons.get_cdr();
    }

    mem.get_global(&key.get().as_symbol().get_name())
}


struct AtomFrame {
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
    car: GcRef,
    cdr: GcRef,
    progress: ConsProgress,
    environment: GcRef,
}


struct ListFrame {
    macro_expand: bool,
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
    fn new(tree: GcRef, environment: GcRef) -> Self {
        if let Some(vec) = list_to_vec(tree.clone()) {
            // tree is a list
            Self::List(ListFrame{ macro_expand: false, eval_args: false, elems: vec, current: 0, environment, in_call: false })
        }
        else if let PrimitiveValue::Cons(cons) = tree.get() {
            // tree is a conscell but not a list
            // (a conscell is a list if its cdr is either a list or nil)
            Self::Cons(ConsFrame{ car: cons.get_car(), cdr: cons.get_cdr(), progress: ConsProgress::NotStartedYet, environment })
        }
        else {
            // tree is an atom
            Self::Atom(AtomFrame{ value: tree, environment, in_call: false })
        }
    }
}


enum EvalInternal {
    Return(GcRef),
    Call(GcRef, GcRef),
    TailCall(GcRef, GcRef),
    Signal(GcRef),
    Abort(String),
}


fn eval_atom(mem: &mut Memory, atom_frame: &mut AtomFrame, return_value: GcRef) -> EvalInternal {
    if atom_frame.in_call {
        // The callee was the normal body of a trap.
        // It has already set the return value, here we just forward it.
        return EvalInternal::Return(return_value);
    }
    
    let atom = atom_frame.value.clone();
    let env  = atom_frame.environment.clone();

    match atom.get() {
        PrimitiveValue::Symbol(_) => {
            if let Some(value) = lookup(mem, atom, env) {
                EvalInternal::Return(value)
            }
            else {
                EvalInternal::Signal(mem.symbol_for("unbound-symbol"))
            }
        },
        PrimitiveValue::Cons(_) => {
            unreachable!("eval_atom received a conscell, but conscells shuld be processed in eval_cons")
        },
        PrimitiveValue::Trap(trap) => {
            atom_frame.in_call = true;
            EvalInternal::Call(trap.get_normal_body(), env)
        },
        _ => {
            // numbers, characters and functions in non-call position don't get evaluated
            EvalInternal::Return(atom)
        },
    }
}


fn eval_cons(mem: &mut Memory, cons_frame: &mut ConsFrame, return_value: GcRef) -> EvalInternal {
    match cons_frame.progress {
        ConsProgress::NotStartedYet => {
            let car = cons_frame.car.clone();
            let env = cons_frame.environment.clone();
            cons_frame.progress = ConsProgress::EvalingCar;
            EvalInternal::Call(car, env)
        },
        ConsProgress::EvalingCar => {
            cons_frame.car = return_value.clone();

            let cdr = cons_frame.cdr.clone();
            let env = cons_frame.environment.clone();
            cons_frame.progress = ConsProgress::EvalingCdr;
            EvalInternal::Call(cdr, env)
        },
        ConsProgress::EvalingCdr => {
            cons_frame.cdr = return_value.clone();
            EvalInternal::Return(mem.allocate_cons(cons_frame.car.clone(), cons_frame.cdr.clone()))
        },
    }
}


fn eval_list(mem: &mut Memory, list_frame: &mut ListFrame, return_value: GcRef) -> EvalInternal {
    // empty list evaluates to nil
    if list_frame.elems.len() == 0 {
        return EvalInternal::Return(GcRef::nil());
    }

    // receive the evaluated element and step to the next one
    if list_frame.in_call {
        list_frame.elems[list_frame.current] = return_value.clone();
        list_frame.current += 1;
        list_frame.in_call = false;
    }

    // the operator has just been evaluated, now we decide whether to evaluate the arguments
    if list_frame.current == 1 {
        if let PrimitiveValue::Function(f) = list_frame.elems[0].get() {
            match f.get_kind() {
                FunctionKind::Lambda        => list_frame.eval_args = true,
                FunctionKind::SpecialLambda => list_frame.eval_args = false,
                _                           => {
                    // cannot evaluate macros at runtime
                    return EvalInternal::Signal(mem.symbol_for("eval-bad-operator"));
                },
            }
        }
        else {
            // first element of list is not a function
            return EvalInternal::Signal(mem.symbol_for("eval-bad-operator"));
        }
    }

    let top =
    if list_frame.eval_args {
        list_frame.elems.len()
    }
    else {
        1
    };

    // evaluate the operator and maybe the operands
    let i = list_frame.current;
    if i < top {
        let x              = list_frame.elems[i].clone();
        list_frame.in_call = true;
        let env            = list_frame.environment.clone();
        return EvalInternal::Call(x, env);
    }

    let operator    = list_frame.elems[0].get().as_function();

    match operator {
        Function::NativeFunction(nf) => {
            match nf.call(mem, &list_frame.elems[1..], list_frame.environment.clone()) {
                NativeResult::Value(x)       => return EvalInternal::Return(x),
                NativeResult::Signal(signal) => return EvalInternal::Signal(signal),
                NativeResult::Abort(msg)     => return EvalInternal::Abort(msg),
            };
        },
        Function::NormalFunction(nf) => {
            // pair the formal parameters with the (possibly evaluated) arguments
            let mut new_env = nf.get_env();
            let mut i = 1; // list_frame.elems[0] is the operator
            for param in nf.params() {
                let arg; 
                if let Some(a) = list_frame.elems.get(i) {
                    arg = a.clone();
                }
                else {
                    return EvalInternal::Signal(mem.symbol_for("not-enough-arguments"));
                };
                let param_arg = mem.allocate_cons(param, arg);
                new_env       = mem.allocate_cons(param_arg, new_env);

                i += 1;
            }

            if i < list_frame.elems.len() {
                return EvalInternal::Signal(mem.symbol_for("too-many-arguments"));
            }

            let new_tree = nf.get_body();
            EvalInternal::TailCall(new_tree, new_env)
        },
    }
}


fn unwind_stack(mem: &mut Memory, stack: &mut Vec<StackFrame>, signal: GcRef) -> Option<StackFrame> {
    while let Some(old_frame) = stack.pop() {
        if let StackFrame::Atom(old_atom_frame) = old_frame {
            if let PrimitiveValue::Trap(trap) = old_atom_frame.value.get() {
                let trap_body          = trap.get_trap_body();
                let mut trap_env       = old_atom_frame.environment;
                let trapped_signal_sym = mem.symbol_for("*trapped-signal*");
                let key_value          = mem.allocate_cons(trapped_signal_sym, signal);
                trap_env               = mem.allocate_cons(key_value, trap_env);
                return Some(StackFrame::new(trap_body, trap_env));
            }
        }
    }

    None
}


fn eval_internal(mem: &mut Memory, tree: GcRef, env: GcRef) -> NativeResult {
    let mut stack        = vec![StackFrame::new(tree, env.clone())];
    let mut return_value = GcRef::nil();

    while let Some(frame) = stack.last_mut() {
        let evaled =
        match frame {
            StackFrame::Atom(ref mut atom_frame) => {
                eval_atom(mem, atom_frame, return_value.clone())
            },
            StackFrame::Cons(ref mut cons_frame) => {
                eval_cons(mem, cons_frame, return_value.clone())
            },
            StackFrame::List(ref mut list_frame) => {
                eval_list(mem, list_frame, return_value.clone())
            },
        };

        match evaled {
            EvalInternal::Return(x) => {
                return_value = x;
            },
            EvalInternal::Call(new_tree, new_env) => {
                stack.push(StackFrame::new(new_tree, new_env));
                continue;
            },
            EvalInternal::TailCall(new_tree, new_env) => {
                *frame = StackFrame::new(new_tree, new_env);
                continue;
            },
            EvalInternal::Signal(signal) => {
                if let Some(trap_frame) = unwind_stack(mem, &mut stack, signal.clone()) {
                    stack.push(trap_frame);
                    continue;
                }
                else {
                    let abort_msg = format!("Unhandled signal: {}", list_to_string(print(mem, &[signal], env.clone()).unwrap()).unwrap());
                    return NativeResult::Abort(abort_msg);
                }
            },
            EvalInternal::Abort(msg) => {
                return NativeResult::Abort(msg);
            },
        }

        stack.pop();

        if !return_value.is_nil() {
            if let PrimitiveValue::Trap(_) = return_value.get() {
                stack.push(StackFrame::new(return_value.clone(), env.clone()));
            }
        }
    }

    NativeResult::Value(return_value)
}


pub fn eval(mem: &mut Memory, args: &[GcRef], env: GcRef) -> NativeResult {
    // TODO: check for too many arguments
    if let Some(tree) = args.first() {
        eval_internal(mem, tree.clone(), env)
    }
    else {
        let signal = mem.symbol_for("wrong-number-of-arguments");
        NativeResult::Signal(signal)
    }
}


pub fn eval_external(mem: &mut Memory, tree: GcRef) -> Result<GcRef, String> {
    let empty_env = GcRef::nil();
    
    match eval(mem, &[tree], empty_env.clone()) {
        NativeResult::Value(x)       => Ok(x),
        NativeResult::Signal(signal) => Err(format!("Unhandled signal: {}", list_to_string(print(mem, &[signal], empty_env).unwrap()).unwrap())),
        NativeResult::Abort(msg)     => Err(msg),
    }
}


#[cfg(test)]
mod tests;
