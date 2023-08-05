use crate::memory::*;
use crate::util::*;
use crate::native::print::print;


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


#[derive(PartialEq, Eq, Clone, Copy)]
enum ListKind {
    BadOperator,
    Empty,
    Lambda,
    SpecialLambda,
}


struct ListFrame {
    kind: ListKind,
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
    fn new(x: GcRef, environment: GcRef) -> Self {
        if let Some(vec) = list_to_vec(x.clone()) {
            let kind =
            if let Some(x) = vec.first() {
                if let PrimitiveValue::Function(f) = x.get() {
                    match f.get_kind() {
                        FunctionKind::Lambda        => ListKind::Lambda,
                        FunctionKind::SpecialLambda => ListKind::SpecialLambda,
                        _                           => ListKind::BadOperator,
                    }
                }
                else {
                    ListKind::BadOperator
                }
            }
            else {
                ListKind::Empty
            };
            Self::List(ListFrame{ kind, elems: vec, current: 0, environment, in_call: false })
        }
        else if let PrimitiveValue::Cons(cons) = x.get() {
            Self::Cons(ConsFrame{ car: cons.get_car(), cdr: cons.get_cdr(), progress: ConsProgress::NotStartedYet, environment })
        }
        else {
            Self::Atom(AtomFrame{ value: x, environment, in_call: false })
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


fn eval_atom(mem: &mut Memory, atom_frame: &mut AtomFrame, return_value: GcRef) -> EvalInternal {
    if atom_frame.in_call {
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
            unreachable!("eval_atom received a conscell, but conscells shuld be processed in eval_internal")
        },
        PrimitiveValue::Trap(trap) => {
            atom_frame.in_call = true;
            EvalInternal::Call(trap.get_normal_body(), env)
        },
        _ => {
            EvalInternal::Return(atom)
        },
    }
}

fn eval_list(mem: &mut Memory, list_frame: &mut ListFrame, return_value: GcRef) -> EvalInternal {
    match list_frame.kind {
        ListKind::Empty => {
            EvalInternal::Return(GcRef::nil())
        },
        ListKind::BadOperator => {
            EvalInternal::Signal(mem.symbol_for("bad-operator"))
        },
        ListKind::Lambda | ListKind::SpecialLambda => {
            if list_frame.in_call {
                list_frame.elems[list_frame.current] = return_value.clone();
                list_frame.current += 1;
                list_frame.in_call = false;
            }

            let i = list_frame.current;
            let top =
            if list_frame.kind == ListKind::Lambda {
                // if lambda then evaluate the operator and the operands
                list_frame.elems.len()
            }
            else {
                // if special-lambda then only evaluate the operator
                1
            };

            if i < top {
                let x = list_frame.elems[i].clone();
                list_frame.in_call = true;
                let env = list_frame.environment.clone();
                return EvalInternal::Call(x, env);
            }

            let mut new_env = list_frame.environment.clone();
            let operator    = list_frame.elems[0].get().as_function();
            for (i, param) in operator.params().enumerate() {
                let arg       = list_frame.elems[i + 1].clone();
                let param_arg = mem.allocate_cons(param, arg);
                new_env       = mem.allocate_cons(param_arg, new_env);
            }

            let new_tree = operator.get_body();
            EvalInternal::TailCall(new_tree, new_env)
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


fn unwind_stack(mem: &mut Memory, stack: &mut Vec<StackFrame>, signal: GcRef) -> Option<StackFrame> {
    while let Some(old_frame) = stack.pop() {
        if let StackFrame::Atom(old_atom_frame) = old_frame {
            if let PrimitiveValue::Trap(trap) = old_atom_frame.value.get() {
                let trap_body          = trap.get_trap_body();
                let mut trap_env       = old_atom_frame.environment;
                let trapped_signal_sym = mem.symbol_for("trapped-signal");
                let key_value          = mem.allocate_cons(trapped_signal_sym, signal);
                trap_env               = mem.allocate_cons(key_value, trap_env);
                return Some(StackFrame::new(trap_body, trap_env));
            }
        }
    }

    None
}


fn eval_internal(mem: &mut Memory, tree: GcRef, environment: GcRef) -> Result<GcRef, String> {
    let mut stack        = vec![StackFrame::new(tree, environment)];
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
                    let abort_msg = format!("Unhandled signal: {}", list_to_string(print(mem, signal)).unwrap());
                    return Err(abort_msg);
                }
            },
            EvalInternal::Abort(msg) => {
                return Err(msg);
            },
        }

        stack.pop();
    }

    Ok(return_value)
}


pub fn eval(mem: &mut Memory, tree: GcRef) -> Result<GcRef, String> {
    let empty_env = GcRef::nil();
    eval_internal(mem, tree, empty_env)
}


#[cfg(test)]
mod tests;