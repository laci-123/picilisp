use pretty_assertions::assert_eq;
use super::*;
use crate::native::print::print;
use crate::util::list_to_string;


#[test]
fn lookup_empty() {
    let mut mem = Memory::new();

    let env = GcRef::nil();
    let key = mem.symbol_for("bird");
    let value = lookup(&mut mem, key, env);
    assert!(value.is_none());
}

#[test]
fn lookup_not_found() {
    let mut mem = Memory::new();

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(10);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(20);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("bird");
    let value = lookup(&mut mem, key, env);
    assert!(value.is_none());
}

#[test]
fn lookup_found() {
    let mut mem = Memory::new();

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(10);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(20);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("falcon");
    let value = lookup(&mut mem, key, env);
    assert_eq!(*value.unwrap().get().unwrap().as_number(), 20);
}

#[test]
fn lookup_global() {
    let mut mem = Memory::new();

    let v0 = mem.allocate_number(00);
    mem.define_global("starling", v0);

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(10);
    let k2 = mem.symbol_for("falcon");
    let v2 = mem.allocate_number(20);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("starling");
    let value = lookup(&mut mem, key, env);
    assert_eq!(*value.unwrap().get().unwrap().as_number(), 00);
}

#[test]
fn lookup_shadowing() {
    let mut mem = Memory::new();

    let v0 = mem.allocate_number(00);
    mem.define_global("starling", v0);

    let k1 = mem.symbol_for("owl");
    let v1 = mem.allocate_number(10);
    let k2 = mem.symbol_for("starling");
    let v2 = mem.allocate_number(20);
    let vec = vec![mem.allocate_cons(k1, v1), mem.allocate_cons(k2, v2)];
    let env = vec_to_list(&mut mem, &vec);
    let key = mem.symbol_for("starling");
    let value = lookup(&mut mem, key, env);
    assert_eq!(*value.unwrap().get().unwrap().as_number(), 20);
}

#[test]
fn make_lambda() {
    let mut mem = Memory::new();

    // (lambda (x y) y)
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![mem.symbol_for("lambda"), vec_to_list(&mut mem, &params), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.clone().unwrap().get().unwrap().as_function().as_normal_function().get_kind(), FunctionKind::Lambda);
    assert_eq_symbol!(value.clone().unwrap().get().unwrap().as_function().as_normal_function().get_body(), mem.symbol_for("y"));
    let p = value.clone().unwrap().get().unwrap().as_function().as_normal_function().non_rest_params().collect::<Vec<GcRef>>();
    assert_eq_symbol!(p[0], mem.symbol_for("x"));
    assert_eq_symbol!(p[1], mem.symbol_for("y"));
}

#[test]
fn make_lambda_bad_param_list() {
    let mut mem = Memory::new();

    // (lambda x y)
    let vec    = vec![mem.symbol_for("lambda"), mem.symbol_for("x"), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind wrong-argument-type source lambda expected list-type actual symbol-type)");
}

#[test]
fn make_lambda_bad_param() {
    let mut mem = Memory::new();

    // (lambda (1) x)
    let params = vec![mem.allocate_number(10)];
    let vec    = vec![mem.symbol_for("lambda"), vec_to_list(&mut mem, &params), mem.symbol_for("x")];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind param-is-not-symbol source lambda param 10)");
}

#[test]
fn make_lambda_not_enough_args() {
    let mut mem = Memory::new();

    // (lambda (x y))
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![mem.symbol_for("lambda"), vec_to_list(&mut mem, &params)];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind wrong-number-of-arguments source lambda expected 2 actual 1)");
}

#[test]
fn make_lambda_too_many_args() {
    let mut mem = Memory::new();

    // (lambda (x y) x y)
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![mem.symbol_for("lambda"), vec_to_list(&mut mem, &params), mem.symbol_for("x"), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind wrong-number-of-arguments source lambda expected 2 actual 3)");
}

#[test]
fn eval_lambda() {
    let mut mem = Memory::new();

    // ((lambda (x y) y) 1 2)
    let params     = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec        = vec![mem.symbol_for("lambda"), vec_to_list(&mut mem, &params), mem.symbol_for("y")];
    let operator   = vec_to_list(&mut mem, &vec);
    let vec2       = vec![operator, mem.allocate_number(10), mem.allocate_number(20)];
    let tree       = vec_to_list(&mut mem, &vec2);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().unwrap().as_number(), 20);
}

#[test]
fn eval_not_lambda() {
    let mut mem = Memory::new();

    let not_lambda = mem.symbol_for("mu");

    // ((mu (x y) y) 1 2)
    let params     = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec        = vec![not_lambda, vec_to_list(&mut mem, &params), mem.symbol_for("y")];
    let operator   = vec_to_list(&mut mem, &vec);
    let vec2       = vec![operator, mem.allocate_number(10), mem.allocate_number(20)];
    let tree       = vec_to_list(&mut mem, &vec2);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind unbound-symbol source eval symbol mu)");
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

    let tree  = mem.allocate_number(3650);
    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().unwrap().as_number(), 3650);
}

#[test]
fn eval_character() {
    let mut mem = Memory::new();

    let tree  = mem.allocate_character('Đ');
    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().unwrap().as_character(), 'Đ');
}

#[test]
fn eval_unbound_symbol() {
    let mut mem = Memory::new();

    let tree  = mem.symbol_for("apple-tree");
    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind unbound-symbol source eval symbol apple-tree)");
}

#[test]
fn eval_cons() {
    let mut mem = Memory::new();

    let x     = mem.allocate_number(10);
    let y     = mem.allocate_number(20);
    let tree  = mem.allocate_cons(x, y);
    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.clone().unwrap().get().unwrap().as_conscell().get_car().get().unwrap().as_number(), 10);
    assert_eq!(*value.unwrap().get().unwrap().as_conscell().get_cdr().get().unwrap().as_number(), 20);
}

#[test]
fn eval_global() {
    let mut mem = Memory::new();

    let x = mem.allocate_number(271);
    mem.define_global("g", x);

    let tree  = mem.symbol_for("g");
    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil(), 0).ok().unwrap()).unwrap();
    assert_eq!(value_str, "271");
}

#[test]
fn eval_global_nil() {
    let mut mem = Memory::new();

    mem.define_global("g", GcRef::nil());

    let tree  = mem.symbol_for("g");
    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil(), 0).ok().unwrap()).unwrap();
    assert_eq!(value_str, "()");
}

#[test]
fn eval_list_bad_operator() {
    let mut mem = Memory::new();

    let vec   = vec![mem.allocate_number(0), mem.allocate_number(-10), mem.allocate_number(-20), mem.allocate_number(-30)];
    let tree  = vec_to_list(&mut mem, &vec);
    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind eval-bad-operator source eval symbol 0)");
}

#[test]
fn eval_call_lambda() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let has_rest_params = false;
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, has_rest_params, body,&params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A'), mem.allocate_character('B')];
    let tree    = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil(), 0).ok().unwrap()).unwrap();
    assert_eq!(value_str, "%B");
}

#[test]
fn eval_call_lambda_unbound_params() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let has_rest_params = false;
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, has_rest_params, body,&params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A'), mem.symbol_for("no-value")];
    let tree    = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind unbound-symbol source eval symbol no-value)");
}

#[test]
fn eval_trap_without_signal() {
    let mut mem = Memory::new();

    let normal_body = mem.allocate_number(100);
    let trap_body   = mem.allocate_character('x');
    let tree        = mem.allocate_trap(normal_body, trap_body);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().unwrap().as_number(), 100);
}

#[test]
fn eval_trap_with_signal() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let has_rest_params = false;
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, has_rest_params, body,&params, GcRef::nil());

    let vec     = vec![lambda, mem.symbol_for("not-bound"), mem.symbol_for("symbols")];
    let normal  = vec_to_list(&mut mem, &vec);

    let trap    = mem.symbol_for("*trapped-signal*");

    let tree    = mem.allocate_trap(normal, trap);

    let value = eval_external(&mut mem, tree);
    let value_str = list_to_string(print(&mut mem, &[value.unwrap()], GcRef::nil(), 0).ok().unwrap()).unwrap();
    assert!(value_str.contains("unbound-symbol"));
}

// receives two arguments, returns the second one
fn test_native_function(mem: &mut Memory, args: &[GcRef], _env: GcRef, _recursion_depth: usize) -> Result<GcRef, GcRef> {
    if args.len() == 2 {
        Ok(args[1].clone())
    }
    else {
        let error = make_error(mem, "wrong-number-of-arguments", "test_native_function", &vec![]);
        Err(error)
    }
}

#[test]
fn eval_native_function_not_enough_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, vec!["x".to_string(), "y".to_string()], test_native_function, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_character('c')];
    let tree = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind wrong-number-of-arguments source test_native_function)");
}

#[test]
fn eval_native_function_too_many_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, vec!["x".to_string(), "y".to_string()], test_native_function, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_character('a'), mem.allocate_character('b'), mem.allocate_character('c')];
    let tree = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind wrong-number-of-arguments source test_native_function)");
}

#[test]
fn eval_native_function() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, vec!["x".to_string(), "y".to_string()], test_native_function, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_number(-123), mem.allocate_number(190)];
    let tree = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.ok().unwrap().get().unwrap().as_number(), 190);
}

#[test]
fn eval_native_eval() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::Lambda, vec!["x".to_string(), "y".to_string()], eval, GcRef::nil());

    let vec  = vec![lambda, mem.allocate_number(-123)];
    let tree = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().unwrap().as_number(), -123);
}

#[test]
fn eval_not_enough_args() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let has_rest_params = false;
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, has_rest_params, body,&params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A')];
    let tree    = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind wrong-number-of-arguments source #<function> expected 2 actual 1)");
}

#[test]
fn eval_too_many_args() {
    let mut mem = Memory::new();

    // a lambda that returns its second parameter
    let params  = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let body    = mem.symbol_for("y");
    let has_rest_params = false;
    let lambda  = mem.allocate_normal_function(FunctionKind::Lambda, has_rest_params, body,&params, GcRef::nil());

    let vec     = vec![lambda, mem.allocate_character('A'), mem.allocate_character('B'), mem.allocate_character('C')];
    let tree    = vec_to_list(&mut mem, &vec);

    let value     = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "(kind wrong-number-of-arguments source #<function> expected 2 actual 3)");
}
