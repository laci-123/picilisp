use assert_cmd::*;
use predicates::{*, prelude::PredicateBooleanExt};



fn check(input: &str, output: &str) {
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["command", input])
                                  .assert().stdout(format!("{output}\n"));
}

fn check_error(input: &str, error_kind: &str, error_details: &str) {
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["command", input])
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
    check_error("2.0", "unbound-symbol", "symbol 2.0"); // TODO: should be syntax error
}

#[test]
fn character_literals() {
    check("%a", "%a");
    check("%ő", "%ő");
    // check("%\n", "%\n"); TODO!
    // check("%\t", "%\t"); TODO!
    check(r"%\\", r"%\\");
    // check("%%", "%%"); TODO!
    check("%猫", "%猫");
    check_error("%abc", "syntax-error", "invalid character: '%abc'");
}
