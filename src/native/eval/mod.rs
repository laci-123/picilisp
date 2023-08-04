use crate::memory::*;
use crate::util::*;
use crate::native::print::print;


struct Atom {
    value: ExternalReference,
    environment: ExternalReference,
    in_call: bool,
}


enum ConsProgress {
    NotStartedYet,
    EvalingCar,
    EvalingCdr,
}


struct Cons {
    car: ExternalReference,
    cdr: ExternalReference,
    progress: ConsProgress,
    environment: ExternalReference,
}


#[derive(PartialEq, Eq, Clone, Copy)]
enum ListKind {
    BadOperator,
    Empty,
    Lambda,
    SpecialLambda,
}


struct List {
    kind: ListKind,
    elems: Vec<ExternalReference>,
    current: usize,
    environment: ExternalReference,
    in_call: bool,
}


enum StackFrame {
    Atom(Atom),
    Cons(Cons),
    List(List),
}

impl StackFrame {
    fn new(x: ExternalReference, environment: ExternalReference) -> Self {
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
            Self::List(List{ kind, elems: vec, current: 0, environment, in_call: false })
        }
        else if let PrimitiveValue::Cons(cons) = x.get() {
            Self::Cons(Cons{ car: cons.get_car(), cdr: cons.get_cdr(), progress: ConsProgress::NotStartedYet, environment })
        }
        else {
            Self::Atom(Atom{ value: x, environment, in_call: false })
        }
    }
}


enum EvalInternal {
    Return(ExternalReference),
    Call(ExternalReference, ExternalReference),
    Signal(ExternalReference),
    Abort(String),
}


fn lookup(key: ExternalReference, environment: ExternalReference) -> Option<ExternalReference> {
    let mut cursor = environment;

    while !cursor.is_nil() {
        let cons = cursor.get().as_conscell();
        let key_value = cons.get_car();

        if key_value.get().as_conscell().get_car().get().as_symbol() == key.get().as_symbol() {
            return Some(key_value.get().as_conscell().get_cdr());
        }

        cursor = cons.get_cdr();
    }

    return None;
}


fn eval_atom(mem: &mut Memory, atom: ExternalReference, environment: ExternalReference) -> EvalInternal {
    match atom.get() {
        PrimitiveValue::Symbol(_) => {
            if let Some(value) = lookup(atom, environment) {
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
            EvalInternal::Call(trap.get_normal_body(), environment)
        },
        _ => {
            EvalInternal::Return(atom)
        },
    }
}


fn unwind_stack(stack: &mut Vec<StackFrame>) -> Option<StackFrame> {
    while let Some(old_frame) = stack.pop() {
        if let StackFrame::Atom(old_atom_frame) = old_frame {
            if let PrimitiveValue::Trap(trap) = old_atom_frame.value.get() {
                let trap_body = trap.get_trap_body();
                let trap_env  = old_atom_frame.environment; // TODO: put `signal` into `trap_env`
                return Some(StackFrame::new(trap_body, trap_env));
            }
        }
    }

    None
}


fn eval_internal(mem: &mut Memory, tree: ExternalReference, environment: ExternalReference) -> EvalInternal {
    let mut stack        = vec![StackFrame::new(tree, environment)];
    let mut return_value = ExternalReference::nil();

    'stack_loop: while let Some(frame) = stack.last_mut() {
        match frame {
            StackFrame::Atom(atom_frame) => {
                if atom_frame.in_call {
                    // coming back from the normal body of a trap, nothing to do here
                }
                else {
                    match eval_atom(mem, atom_frame.value.clone(), atom_frame.environment.clone()) {
                        EvalInternal::Return(x) => {
                            return_value = x;
                        },
                        EvalInternal::Call(new_tree, new_env) => {
                            atom_frame.in_call = true;
                            stack.push(StackFrame::new(new_tree, new_env));
                            continue 'stack_loop;
                        },
                        EvalInternal::Signal(signal) => {
                            if let Some(trap_frame) = unwind_stack(&mut stack) {
                                stack.push(trap_frame);
                                continue 'stack_loop;
                            }
                            else {
                                return EvalInternal::Signal(signal);
                            }
                        },
                        EvalInternal::Abort(msg) => {
                            return EvalInternal::Abort(msg);
                        },
                    }
                }
            },
            StackFrame::Cons(cons_frame) => {
                match cons_frame.progress {
                    ConsProgress::NotStartedYet => {
                        let car = cons_frame.car.clone();
                        let env = cons_frame.environment.clone();
                        cons_frame.progress = ConsProgress::EvalingCar;
                        stack.push(StackFrame::new(car, env));
                        continue 'stack_loop;
                    },
                    ConsProgress::EvalingCar => {
                        cons_frame.car = return_value.clone();

                        let cdr = cons_frame.cdr.clone();
                        let env = cons_frame.environment.clone();
                        cons_frame.progress = ConsProgress::EvalingCdr;
                        stack.push(StackFrame::new(cdr, env));

                        continue 'stack_loop;
                    },
                    ConsProgress::EvalingCdr => {
                        cons_frame.cdr = return_value.clone();
                        return_value = mem.allocate_cons(cons_frame.car.clone(), cons_frame.cdr.clone());
                    },
                }
            },
            StackFrame::List(list_frame) => {
                match list_frame.kind {
                    ListKind::Empty => {
                        return_value = ExternalReference::nil();
                    },
                    ListKind::BadOperator => {
                        let signal = mem.symbol_for("bad-operator");

                        if let Some(trap_frame) = unwind_stack(&mut stack) {
                            stack.push(trap_frame);
                            continue 'stack_loop;
                        }
                        else {
                            return EvalInternal::Signal(signal);
                        }
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
                            stack.push(StackFrame::new(x, env));
                            continue 'stack_loop;
                        }

                        let mut new_env = list_frame.environment.clone();
                        let operator    = list_frame.elems[0].get().as_function();
                        for (i, param) in operator.params().enumerate() {
                            let arg       = list_frame.elems[i + 1].clone();
                            let param_arg = mem.allocate_cons(param, arg);
                            new_env       = mem.allocate_cons(param_arg, new_env);
                        }

                        let new_tree = operator.get_body();
                        *frame = StackFrame::new(new_tree, new_env);

                        continue 'stack_loop;
                    },
                }
            },
        }

        stack.pop();
    }

    EvalInternal::Return(return_value)
}


pub fn eval(mem: &mut Memory, tree: ExternalReference) -> Result<ExternalReference, String> {
    let empty_env = ExternalReference::nil();
    
    match eval_internal(mem, tree, empty_env) {
        EvalInternal::Return(x)      => Ok(x),
        EvalInternal::Call(_, _)     => unreachable!("`eval_internal` returned `EvalInternal::Call` when called from `eval`"),
        EvalInternal::Signal(signal) => Err(format!("Unhandled signal: {}", list_to_string(print(mem, signal)).unwrap())),
        EvalInternal::Abort(msg)     => Err(msg),
    }
}


#[cfg(test)]
mod tests;
