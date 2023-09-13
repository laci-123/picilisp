use pretty_assertions::assert_eq;
use crate::native::eval::eval_external;
use crate::util::*;
use super::*;



#[test]
fn make_lambda() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda, GcRef::nil());

    // (lambda (x y) y)
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![lambda, vec_to_list(&mut mem, &params), mem.symbol_for("y")];
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

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda, GcRef::nil());

    // (lambda x y)
    let vec    = vec![lambda, mem.symbol_for("x"), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: (kind bad-param-list source lambda)");
}

#[test]
fn make_lambda_bad_param() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda, GcRef::nil());

    // (lambda (1) x)
    let params = vec![mem.allocate_number(10)];
    let vec    = vec![lambda, vec_to_list(&mut mem, &params), mem.symbol_for("x")];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: (kind param-is-not-symbol source lambda param 10)");
}

#[test]
fn make_lambda_not_enough_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda, GcRef::nil());

    // (lambda (x y))
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![lambda, vec_to_list(&mut mem, &params)];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: (kind wrong-number-of-arguments source lambda expected 2 actual 1)");
}

#[test]
fn make_lambda_too_many_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda, GcRef::nil());

    // (lambda (x y) x y)
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![lambda, vec_to_list(&mut mem, &params), mem.symbol_for("x"), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, &vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: (kind wrong-number-of-arguments source lambda expected 2 actual 3)");
}

#[test]
fn eval_lambda() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda, GcRef::nil());

    // ((lambda (x y) y) 1 2)
    let params     = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec        = vec![lambda, vec_to_list(&mut mem, &params), mem.symbol_for("y")];
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
    assert_eq!(value.err().unwrap(), "Unhandled signal: (kind unbound-symbol source eval symbol mu)");
}
