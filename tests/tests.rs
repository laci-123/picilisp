use assert_cmd::*;
use predicates::{*, prelude::PredicateBooleanExt};



fn check(input: &str, output: &str) {
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["--expression", input])
                                  .assert().stdout(format!("{output}\n"));
}

fn check_error(input: &str, error_kind: &str, error_details: &str) {
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["--expression", input])
                                  .assert().stderr(str::contains(format!("kind {error_kind}")).and(str::contains(error_details)));
}


#[test]
fn number_literals() {
    check("1", "1");
    check("1000", "1000");
    check("-3", "-3");
    check("+5", "5");
    check("0", "0");
    check("-0", "0");
    check_error("2.0", "syntax-error", "unexpected character in number literal: '.'");
}

#[test]
fn character_literals() {
    check("%a", "%a");
    check("%ő", "%ő");
    check(r"%\n", r"%\n");
    check(r"%\t", r"%\t");
    check(r"%\s", r"%\s");
    check(r"%\\", r"%\\");
    check("%%", "%%");
    check("%猫", "%猫");
    check_error("%abc", "syntax-error", "invalid character: '%abc'");
}

#[test]
fn symbols_and_quoting() {
    check("(quote abc)", "abc");
    check("(quote -abc*def>!)", "-abc*def>!");
    check("(quote cat%dog)", "cat%dog");
    check("'thing", "thing");
}

#[test]
fn string_literals() {
    check(r#""árvíztűrő tükörfúrógép""#, r#""árvíztűrő tükörfúrógép""#);
    check(r#""first line\nsecond line\n\nfourth line""#, r#""first line
second line

fourth line""#);
    check(r#""something in quotes: \"something\".""#, r#""something in quotes: \"something\".""#); 
    check("(list %a %b %c)", r#""abc""#);
    check_error(r#""ci\ca""#, "syntax-error", "'c' is not a valid escape character in a string literal");
}

#[test]
fn lists() {
    check("()", "()");
    check("(list 1 2 3)", "(1 2 3)");
    check("(list 1 (list 2) () (list 3 4))", "(1 (2) () (3 4))");
    check("(list 1 'cat %A)", "(1 cat %A)");
}

#[test]
fn comments() {
    check(r#"(list 1 ; this is a comment
2 3
4 5 ;; this is an other one with weird symbols: ()'";
)
"#, "(1 2 3 4 5)");
}

#[test]
fn function_literals() {
    let input = "(lambda (x) x)";
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["--expression", input])
                                  .assert().stdout(str::contains("#<lambda-0x"));

    let input = "(macro (x) x)";
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["--expression", input])
                                  .assert().stdout(str::contains("#<macro-0x"));
}

#[test]
fn bad_function_literals() {
    check_error("(lambda x x)", "wrong-argument-type", "expected list-type actual symbol-type");
    check_error("(lambda (x))", "wrong-number-of-arguments", "expected 2 actual 1");
    check_error("(lambda (x y) x y)", "wrong-number-of-arguments", "expected 2 actual 3");
}

#[test]
fn lambdas() {
    check("((lambda (x) x) 1)", "1");
    check("((lambda (x y) x) 1 2)", "1");
    check("((lambda (x y) y) 1 2)", "2");
    check("((lambda (x y & z) z) 1 2 3 4 5)", "(3 4 5)");
    check("((lambda (x y) (add x y)) 1 2)", "3");
}

#[test]
fn closures() {
    check("(((lambda (x) (lambda (y) x)) 1) 2)", "1");
    check("(((lambda (x) (lambda (y) (add x y))) 1) 2)", "3");
}

#[test]
fn macros() {
    check("((macro (x y z) (list z y x)) 2 1 add)", "3");
    check("(macroexpand '((macro (x y z) (list z y x)) 2 1 add))", "(add 1 2)");
}

#[test]
fn branches() {
    check("(if 1 2 3)", "2");
    check("(if () 2 3)", "3");
    check("(if (= 1 1) 'true unquoted-symbol)", "true");
    check("(if (= 1 2) unquoted-symbol 'false)", "false");
}

#[test]
fn traps() {
    check("(eval (trap unquoted-symbol (. *trapped-signal* 'kind)))", "unbound-symbol");
    check("(eval (trap (list (list (list 1 2 deeply-nested 3))) (. *trapped-signal* 'symbol)))", "deeply-nested");
    check("(eval (trap (list 1 2 (signal \"Boo!\") 3) *trapped-signal*))", "\"Boo!\"");
}

#[test]
fn let_macro() {
    check("(let (x 1) x)", "1");
    check("(let (x 1, y 2) (add x y))", "3");
    check_error("(let (x 1, y (add x 1)) y)", "unbound-symbol", "symbol x");
}

#[test]
fn print() {
    check(r#"(print 123)"#, r#""123""#);
    check(r#"(print %🐋)"#, r#""%🐋""#);
    check(r#"(print 'elephant)"#, r#""elephant""#);
    check(r#"(print "this is a string")"#, r#""\"this is a string\"""#); 
    check(r#"(print (list 1 (list %a 'b) () 3))"#, r#""(1 (%a b) () 3)""#);
}

#[test]
fn cons_car_cdr() {
    check("(cons 1 2)", "(cons 1 2)");
    check("(cons 1 ())", "(1)");
    check("(car (cons 1 2))", "1");
    check("(cdr (cons 1 2))", "2");
    check("(car (list 'a 'b 'c))", "a");
    check("(cdr (list 'a 'b 'c))", "(b c)");
    check("(cons 'A (list 'a 'b 'c))", "(A a b c)");
    check("(cdr (list 1))", "()");
}

#[test]
fn equality() {
    check("(= 1 1)", "t");
    check("(= 1 2)", "()");
    check("(= %z %z)", "t");
    check("(= %z %Z)", "()");
    check("(= %z 'z)", "()");
    check("(= () ())", "t");
    check("(= nil ())", "t");
}

#[test]
fn equality_2() {
    check("(= (list 1 2 3) (list 1 2 3))", "t");
    check("(= (list 1 2) (list 1 2 3))", "()");
    check("(= (list 1 2 3) (list 1 2))", "()");
    check("(= (list (list 1 2) () (list (list 1 2 3) 4)) (list (list 1 2) () (list (list 1 2 3) 4)))", "t");
    check("(let (x 1, y 1) (= x y))", "t");
    check("(let (x 1, y 1) (= 'x 'y))", "()");
}

#[test]
fn gensyms() {
    let input = "(gensym)";
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["--expression", input])
                                  .assert().stdout(str::contains("#<symbol-0x"));
    check("(= (gensym) (gensym))", "()");
    check("(let (x (gensym)) (= x x))", "t");
}
