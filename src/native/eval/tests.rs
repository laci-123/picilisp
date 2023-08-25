use pretty_assertions::assert_eq;
use super::*;
use crate::native::print::print;
use crate::util::list_to_string;


#[test]
fn lookup_empty() {
    let mut mem = Memory::new();

    let env = GcRef::nil();
    let key = mem.symbol_for("bird");
    let value = lookup(&mem, key, env);
    assert!(value.is_none());
}

#[test]
fn lookup_not_found() {
    let mut mem = Memory::new();

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(1.0);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(2.0);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("bird");
    let value = lookup(&mem, key, env);
    assert!(value.is_none());
}

#[test]
fn lookup_found() {
    let mut mem = Memory::new();

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(1.0);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(2.0);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("falcon");
    let value = lookup(&mem, key, env);
    assert_eq!(*value.unwrap().get().as_number(), 2.0);
}

#[test]
fn lookup_global() {
    let mut mem = Memory::new();

    let v0 = mem.allocate_number(0.0);
    mem.define_global("starling", v0);

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(1.0);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(2.0);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("starling");
    let value = lookup(&mem, key, env);
    assert_eq!(*value.unwrap().get().as_number(), 0.0);
}

#[test]
fn lookup_shadowing() {
    let mut mem = Memory::new();

    let v0 = mem.allocate_number(0.0);
    mem.define_global("starling", v0);

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(1.0);
    let k2 = mem.symbol_for("starling");
    let v2 = mem.allocate_number(2.0);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("starling");
    let value = lookup(&mem, key, env);
    assert_eq!(*value.unwrap().get().as_number(), 2.0);
}

#[test]
fn eval_nil() {
    let mut mem = Memory::new();

    let tree  = GcRef::nil();
    let value = eval_external(&mut mem, tree);
    assert!(value.unwrap().is_nil());
}

#[test]
fn eval_number() {
    let mut mem = Memory::new();

    let tree  = mem.allocate_number(365.0);
    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_number(), 365.0);
}

#[test]
fn eval_character() {
    let mut mem = Memory::new();

    let tree  = mem.allocate_character('Đ');
    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_character(), 'Đ');
}

#[test]
fn eval_unbound_symbol() {
    let mut mem = Memory::new();

    let tree  = mem.symbol_for("apple-tree");
    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: unbound-symbol");
}

#[test]
fn eval_cons() {
    let mut mem = Memory::new();

    let x     = mem.allocate_number(1.0);
    let y     = mem.allocate_number(2.0);
    let tree  = mem.allocate_cons(x, y);
    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.clone().unwrap().get().as_conscell().get_car().get().as_number(), 1.0);
    assert_eq!(*value.unwrap().get().as_conscell().get_cdr().get().as_number(), 2.0);
}

#[test]
fn eval_global() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(2.71);
    mem.define_global("g", x);

    let tree  = mem.symbol_for("g");
    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil()).unwrap()).unwrap();
    assert_eq!(value_str, "2.71");
}

#[test]
fn eval_global_nil() {
    let mut mem = Memory::new();

    mem.define_global("g", GcRef::nil());

    let tree  = mem.symbol_for("g");
    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil()).unwrap()).unwrap();
    assert_eq!(value_str, "()");
}

#[test]
fn eval_list_bad_operator() {
    let mut mem = Memory::new();

    let vec   = vec![mem.allocate_number(0.0), mem.allocate_number(-1.0), mem.allocate_number(-2.0), mem.allocate_number(-3.0)];
    let tree  = vec_to_list(&mut mem, &vec);
    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: eval-bad-operator");
}

#[test]
fn eval_lambda() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, body, params, GcRef::nil());

    let value = eval_external(&mut mem, lambda);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil()).unwrap()).unwrap();
    assert_eq!(value_str, "#<function>");
}

#[test]
fn eval_call_lambda() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, body, params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A'), mem.allocate_character('B')];
    let tree    = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil()).unwrap()).unwrap();
    assert_eq!(value_str, "%B");
}

#[test]
fn eval_call_lambda_unbound_params() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, body, params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A'), mem.symbol_for("no-value")];
    let tree    = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: unbound-symbol");
}

#[test]
fn eval_call_special_lambda() {
    let mut mem = Memory::new();

    // a special-lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_normal_function(FunctionKind::SpecialLambda, body, params, GcRef::nil());

    let vec     = vec![lambda, mem.symbol_for("not-bound"), mem.symbol_for("symbols")];
    let tree    = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil()).unwrap()).unwrap();
    assert_eq!(value_str, "symbols");
}

#[test]
fn eval_trap_without_signal() {
    let mut mem = Memory::new();

    let normal_body = mem.allocate_number(10.0);
    let trap_body   = mem.allocate_character('x');
    let tree        = mem.allocate_trap(normal_body, trap_body);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_number(), 10.0);
}

#[test]
fn eval_trap_with_signal() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, body, params, GcRef::nil());

    let vec     = vec![lambda, mem.symbol_for("not-bound"), mem.symbol_for("symbols")];
    let normal  = vec_to_list(&mut mem, &vec);

    let trap    = mem.symbol_for("*trapped-signal*");

    let tree    = mem.allocate_trap(normal, trap);

    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil()).unwrap()).unwrap();
    assert_eq!(value_str, "unbound-symbol");
}

// receives two arguments, returns the second one
fn test_native_function(mem: &mut Memory, args: &[GcRef], _env: GcRef) -> NativeResult {
    if args.len() == 2 {
        NativeResult::Value(args[1].clone())
    }
    else {
        let signal = mem.symbol_for("wrong-number-of-arguments");
        NativeResult::Signal(signal)
    }
}

#[test]
fn eval_native_function_not_enough_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, test_native_function, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_character('c')];
    let tree = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: wrong-number-of-arguments");
}

#[test]
fn eval_native_function_too_many_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, test_native_function, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_character('a'), mem.allocate_character('b'), mem.allocate_character('c')];
    let tree = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: wrong-number-of-arguments");
}

#[test]
fn eval_native_function() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, test_native_function, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_number(-1.23), mem.allocate_number(19.0)];
    let tree = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.ok().unwrap().get().as_number(), 19.0);
}

#[test]
fn eval_native_eval() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, eval, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_number(-1.23)];
    let tree = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_number(), -1.23);
}

#[test]
fn eval_not_enough_args() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, body, params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A')];
    let tree    = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: not-enough-arguments");
}

#[test]
fn eval_too_many_args() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, body, params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A'), mem.allocate_character('B'), mem.allocate_character('C')];
    let tree    = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: too-many-arguments");
}
