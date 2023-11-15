use assert_cmd::*;



fn check(input: &str, output: &str) {
    Command::cargo_bin("picilisp").unwrap()
                                  .args(&["command", input])
                                  .assert().stdout(format!("{output}\n"));
}

#[test]
fn number_literals() {
    check("1", "1");
    check("1000", "1000");
    check("-3", "-3");
    check("+5", "5");
    check("0", "0");
    check("-0", "0");
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
}
