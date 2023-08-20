use pretty_assertions::assert_eq;
use crate::native::eval::eval_external;
use crate::util::vec_to_list;
use super::*;



#[test]
fn make_lambda() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda);

    // (lambda (x y) y)
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![lambda, vec_to_list(&mut mem, params), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.clone().unwrap().get().as_function().as_normal_function().get_kind(), FunctionKind::Lambda);
    assert_eq!(value.clone().unwrap().get().as_function().as_normal_function().get_body().get().as_symbol(), mem.symbol_for("y").get().as_symbol());
    let p = value.clone().unwrap().get().as_function().as_normal_function().params().collect::<Vec<GcRef>>();
    assert_eq!(p[0].get().as_symbol(), mem.symbol_for("x").get().as_symbol());
    assert_eq!(p[1].get().as_symbol(), mem.symbol_for("y").get().as_symbol());
}

#[test]
fn make_lambda_bad_params() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda);

    // (lambda x y)
    let vec    = vec![lambda, mem.symbol_for("x"), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: bad-param-list");
}

#[test]
fn make_lambda_not_enough_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda);

    // (lambda (x y))
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![lambda, vec_to_list(&mut mem, params)];
    let tree   = vec_to_list(&mut mem, vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: wrong-number-of-arguments");
}

#[test]
fn make_lambda_too_many_args() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda);

    // (lambda (x y) x y)
    let params = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec    = vec![lambda, vec_to_list(&mut mem, params), mem.symbol_for("x"), mem.symbol_for("y")];
    let tree   = vec_to_list(&mut mem, vec);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: wrong-number-of-arguments");
}

#[test]
fn eval_lambda() {
    let mut mem = Memory::new();

    let lambda = mem.allocate_native_function(FunctionKind::SpecialLambda, lambda);

    // ((lambda (x y) y) 1 2)
    let params     = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec        = vec![lambda, vec_to_list(&mut mem, params), mem.symbol_for("y")];
    let operator   = vec_to_list(&mut mem, vec);
    let vec2       = vec![operator, mem.allocate_number(1.0), mem.allocate_number(2.0)];
    let tree       = vec_to_list(&mut mem, vec2);

    let value = eval_external(&mut mem, tree);
    assert_eq!(*value.unwrap().get().as_number(), 2.0);
}

#[test]
fn eval_not_lambda() {
    let mut mem = Memory::new();

    let not_lambda = mem.symbol_for("mu");

    // ((mu (x y) y) 1 2)
    let params     = vec![mem.symbol_for("x"), mem.symbol_for("y")];
    let vec        = vec![not_lambda, vec_to_list(&mut mem, params), mem.symbol_for("y")];
    let operator   = vec_to_list(&mut mem, vec);
    let vec2       = vec![operator, mem.allocate_number(1.0), mem.allocate_number(2.0)];
    let tree       = vec_to_list(&mut mem, vec2);

    let value = eval_external(&mut mem, tree);
    assert_eq!(value.err().unwrap(), "Unhandled signal: unbound-symbol");
}
