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
fn append() {
    check("(append nil nil)", "()");
    check("(append '(1 2 3) nil)", "(1 2 3)");
    check("(append nil '(1 2 3))", "(1 2 3)");
    check("(append '(1 2 3) '(a b c))", "(1 2 3 a b c)");
    check("(append '(1 2 3) \"xyz\")", "(1 2 3 %x %y %z)");
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

#[test]
fn when() {
    check("(when (= 1 1) 'true)", "true");
    check("(when (= 1 2) 'true)", "()");
}

#[test]
fn foldl() {
    check("(foldl (lambda (x y) (cons y x)) nil ())", "()");
    check("(foldl (lambda (x y) (cons y x)) nil (list 1 2 3 4 5))", "(5 4 3 2 1)");
}

#[test]
fn foldr() {
    check("(foldr (lambda (x y) (cons x y)) nil ())", "()");
    check("(foldr (lambda (x y) (cons x y)) nil (list 1 2 3 4 5))", "(1 2 3 4 5)");
}

#[test]
fn reverse() {
    check("(reverse nil)", "()");
    check("(reverse (list 1))", "(1)");
    check("(reverse (list 1 2 3 4 5))", "(5 4 3 2 1)");
}

#[test]
fn zip() {
    check("(zip nil nil)", "()");
    check("(zip '(1 2 3) nil)", "()");
    check("(zip nil '(1 2 3))", "()");
    check("(zip '(a b c) '(1 2 3))", "((cons a 1) (cons b 2) (cons c 3))");
    check("(zip '(a b c d) '(1 2 3))", "((cons a 1) (cons b 2) (cons c 3))");
    check("(zip '(a b c) '(1 2 3 4))", "((cons a 1) (cons b 2) (cons c 3))");
}

#[test]
fn enumerate() {
    check("(enumerate nil)", "()");
    check("(enumerate '(x y z))", "((cons x 0) (cons y 1) (cons z 2))");
}

#[test]
fn map() {
    check("(map (lambda (x) (+ x 1)) nil)", "()");
    check("(map (lambda (x) (+ x 1)) (list 1 2 3 4 5))", "(2 3 4 5 6)");
}

#[test]
fn apply() {
    check("(apply + nil)", "0");
    check("(apply + (list 1 2 3))", "6");
}

#[test]
fn last() {
    check_error("(last nil)", "wrong-argument", "details empty-list");
    check("(last (list 1))", "1");
    check("(last (list 1 2 3 4 5))", "5");
}

#[test]
fn output() {
    check("(output (print 'elephant))", "elephant\nok");
}

#[test]
fn block() {
    check("(block)", "()");
    check("(block 'blue-whale)", "blue-whale");
    check("(block (output (print 'apple)) 123 (output (print 'orange)) 42)", "apple\norange\n42");
}

#[test]
fn and() {
    check("(and t t)", "t");
    check("(and t nil)", "()");
    check("(and nil t)", "()");
    check("(and nil nil)", "()");
    check("(and nil (output \"monkey\"))", "()");
}

#[test]
fn or() {
    check("(or t t)", "t");
    check("(or t nil)", "t");
    check("(or nil t)", "t");
    check("(or nil nil)", "()");
    check("(or t (output \"monkey\"))", "t");
}

#[test]
fn not() {
    check("(not t)", "()");
    check("(not ())", "t");
}

#[test]
fn plus_minus() {
    check("(+)", "0");
    check("(+ 1)", "1");
    check("(+ 1 2 3)", "6");
    check("(-)", "0");
    check("(- 1)", "-1");
    check("(- 1 2)", "-1");
    check("(- 1 2 3)", "-4");
}

#[test]
fn multiply() {
    check("(*)", "1");
    check("(* 2)", "2");
    check("(* 2 10)", "20");
    check("(* 2 3 4 5)", "120");
    check_error("(* 100000000000 100000000000)", "arithmetic-overflow", "");
}

#[test]
fn divide() {
    check("(/)", "1");
    check("(/ 60 12)", "5");
    check_error("(/ 60 0)", "divide-by-zero", "");
}

#[test]
fn range() {
    check("(range 0)", "()");
    check("(range 1)", "(0)");
    check("(range 10)", "(0 1 2 3 4 5 6 7 8 9)");
}

#[test]
fn length() {
    check("(length nil)", "0");
    check("(length '(1))", "1");
    check("(length '(1 2 3))", "3");
    check("(length (range 100))", "100");
}

#[test]
fn concat() {
    check("(concat)", "()");
    check("(concat '(a b c))", "(a b c)");
    check("(concat (list 1 2 3) (list 4 5 6) (list 7 8 9))", "(1 2 3 4 5 6 7 8 9)");
    check("(concat \"The sky is: \" (print 'blue) \".\")", "\"The sky is: blue.\"");
}

#[test]
fn describe() {
    check("(describe print)", "\"(lambda (input) ...)

Convert `input` to its string representation.

Defined in:
 Rust source.\"");

    check("(describe read)", "\"(lambda (input source start-line start-column) ...)

Converts a Lisp-style string to an AST.

Only reads the shortest prefix of the input string that is a valid AST.

Returns a property list which always contains at least a `status` key.
The `status` key can have one of the following values:
 * `ok`:         Success. The key `result` is the AST.
 * `nothing`:    The input was empty or only contained whitespace.
 * `incomplete`: The input is not a valid AST, but can be the beginning of a valid AST.
 * `error`:      The input is not a valid AST, not even the beginning of one. The `error` key contains the error details.
 * `invalid`:    The input is not a valid string.

Whenever there is a `rest` key, the `line` and `column` keys are also present,
whose values are respectively the first line and column of the rest of the input.

`source`, `start-line` and `start-column` describe where we are reading from.
Possible values of `source`:
 * prelude
 * stdin
 * a string representing a file-path.

Defined in:
 Rust source.\"");
}

#[test]
fn read_simple() {
    check("(read-simple \"(a b c) (d e f)\")", "(a b c)");
    check_error("(read-simple \"(a b c\")", "read-error", "details (status incomplete)");
}

#[test]
fn try_catch() {
    check("(try 123 (catch unbound-symbol (lambda (x) (. x 'symbol))) (catch-all (lambda (_) 'something-else)))", "123");
    check("(try something (catch unbound-symbol (lambda (x) (. x 'symbol))) (catch-all (lambda (_) 'something-else)))", "something");
    check("(try (1 2 3) (catch unbound-symbol (lambda (x) (. x 'symbol))) (catch-all (lambda (_) 'something-else)))", "something-else");
}

#[test]
fn metadata() {
    check("(get-metadata (read-simple \"   123\"))", "(documentation () file stdin line 1 column 4)");
}


#[test]
fn defun() {
    check("(block (defun f (x y) \"\" (+ x y)) (f 1 2))", "3");
}

#[test]
fn recursion() {
    check("(block (defun factorial (n) \"\" (if (= n 0) 1 (* n (factorial (- n 1))))) (factorial 5))", "120");
}
