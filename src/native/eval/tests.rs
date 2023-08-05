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
    let env = vec_to_list(&mut mem, vec);
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
    let env = vec_to_list(&mut mem, vec);
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
    let env = vec_to_list(&mut mem, vec);
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
    let env = vec_to_list(&mut mem, vec);
    let key = mem.symbol_for("starling");
    let value = lookup(&mem, key, env);
    assert_eq!(*value.unwrap().get().as_number(), 2.0);
}

#[test]
fn eval_nil() {
    let mut mem = Memory::new();

    let tree  = GcRef::nil();
    let value = eval(&mut mem, tree);
    assert!(value.unwrap().is_nil());
}

#[test]
fn eval_number() {
    let mut mem = Memory::new();

    let tree  = mem.allocate_number(365.0);
    let value = eval(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_number(), 365.0);
}

#[test]
fn eval_character() {
    let mut mem = Memory::new();

    let tree  = mem.allocate_character('Đ');
    let value = eval(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_character(), 'Đ');
}

#[test]
fn eval_unbound_symbol() {
    let mut mem = Memory::new();

    let tree  = mem.symbol_for("apple-tree");
    let value = eval(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: unbound-symbol");
}

#[test]
fn eval_cons() {
    let mut mem = Memory::new();

    let x     = mem.allocate_number(1.0);
    let y     = mem.allocate_number(2.0);
    let tree  = mem.allocate_cons(x, y);
    let value = eval(&mut mem, tree);
    assert_eq!(*value.clone().unwrap().get().as_conscell().get_car().get().as_number(), 1.0);
    assert_eq!(*value.unwrap().get().as_conscell().get_cdr().get().as_number(), 2.0);
}

#[test]
fn eval_list_bad_operator() {
    let mut mem = Memory::new();

    let vec   = vec![mem.symbol_for("not-an-operator"), mem.allocate_number(-1.0), mem.allocate_number(-2.0), mem.allocate_number(-3.0)];
    let tree  = vec_to_list(&mut mem, vec);
    let value = eval(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: bad-operator");
}

#[test]
fn eval_lambda() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_function(false, FunctionKind::Lambda, body, params);

    let value = eval(&mut mem, lambda);
    let value_str = list_to_string(print(&mut mem, value.unwrap())).unwrap();
    assert_eq!(value_str, "#<function>");
}

#[test]
fn eval_call_lambda() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_function(false, FunctionKind::Lambda, body, params);

    let vec     = vec![lambda, mem.allocate_character('A'), mem.allocate_character('B')];
    let tree    = vec_to_list(&mut mem, vec);

    let value = eval(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, value.unwrap())).unwrap();
    assert_eq!(value_str, "%B");
}

#[test]
fn eval_call_lambda_unbound_params() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_function(false, FunctionKind::Lambda, body, params);

    let vec     = vec![lambda, mem.allocate_character('A'), mem.symbol_for("no-value")];
    let tree    = vec_to_list(&mut mem, vec);

    let value = eval(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: unbound-symbol");
}

#[test]
fn eval_call_special_lambda() {
    let mut mem = Memory::new();

    // a special-lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_function(false, FunctionKind::SpecialLambda, body, params);

    let vec     = vec![lambda, mem.symbol_for("not-bound"), mem.symbol_for("symbols")];
    let tree    = vec_to_list(&mut mem, vec);

    let value = eval(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, value.unwrap())).unwrap();
    assert_eq!(value_str, "symbols");
}

#[test]
fn eval_trap_without_signal() {
    let mut mem = Memory::new();

    let normal_body = mem.allocate_number(10.0);
    let trap_body   = mem.allocate_character('x');
    let tree        = mem.allocate_trap(normal_body, trap_body);

    let value = eval(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_number(), 10.0);
}

#[test]
fn eval_trap_with_signal() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let lambda  = mem.allocate_function(false, FunctionKind::Lambda, body, params);

    let vec     = vec![lambda, mem.symbol_for("not-bound"), mem.symbol_for("symbols")];
    let normal  = vec_to_list(&mut mem, vec);

    let trap    = mem.symbol_for("*trapped-signal*");

    let tree    = mem.allocate_trap(normal, trap);

    let value = eval(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, value.unwrap())).unwrap();
    assert_eq!(value_str, "unbound-symbol");
}

#[test]
fn eval_unknown_native_function() {
    let mut mem = Memory::new();

    let params      = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let native_name = mem.symbol_for("no-such-thing");
    let lambda      = mem.allocate_function(true, FunctionKind::Lambda, native_name, params);

    let vec  = vec![lambda, mem.allocate_number(100.0), mem.allocate_number(200.0)];
    let tree = vec_to_list(&mut mem, vec);

    let value = eval(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: unknown-native-function");
}

#[test]
fn eval_native_function() {
    let mut mem = Memory::new();

    let params      = vec![mem.symbol_for("tree")];
    let native_name = mem.symbol_for("print");
    let lambda      = mem.allocate_function(true, FunctionKind::Lambda, native_name, params);

    let vec  = vec![lambda, mem.allocate_number(-1.23)];
    let tree = vec_to_list(&mut mem, vec);

    let value     = eval(&mut mem, tree);
    let value_str = list_to_string(value.unwrap()).unwrap();
    assert_eq!(value_str, "-1.23");
}
