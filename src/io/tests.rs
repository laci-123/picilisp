use pretty_assertions::assert_eq;
use std::io::{Read, Write};
use super::*;



#[test]
fn input_buffer_empty() {
    let mut ib = InputBuffer::new();
    let mut s = String::new();
    let n = ib.read_to_string(&mut s).unwrap();
    assert_eq!(n, 0);
    assert_eq!(s, "");
    assert!(ib.all_is_read());
}

#[test]
fn input_buffer_simple() {
    let mut ib = InputBuffer::new();
    ib.push_string("lion");
    assert!(!ib.all_is_read());
    let mut s = String::new();
    let n = ib.read_to_string(&mut s).unwrap();
    assert_eq!(n, 4);
    assert_eq!(s, "lion");
    assert!(ib.all_is_read());
}

#[test]
fn input_buffer_multiple_reads() {
    let mut ib = InputBuffer::new();
    ib.push_string("elephant");
    assert!(!ib.all_is_read());
    let mut buf = [0 as u8; 3];
    let n = ib.read(&mut buf).unwrap();
    assert_eq!(n, 3);
    assert_eq!(&buf, b"ele");
    assert!(!ib.all_is_read());
    let mut buf2 = [0 as u8; 5];
    let n2 = ib.read(&mut buf2).unwrap();
    assert_eq!(n2, 5);
    assert_eq!(&buf2, b"phant");
    assert!(ib.all_is_read());
}

#[test]
fn input_buffer_unicode() {
    let mut ib = InputBuffer::new();
    ib.push_string("動物園");
    assert!(!ib.all_is_read());
    let mut s = String::new();
    let n = ib.read_to_string(&mut s).unwrap();
    assert_eq!(n, 9);
    assert_eq!(s, "動物園");
    assert!(ib.all_is_read());
}

#[test]
fn input_buffer_clear() {
    let mut ib = InputBuffer::new();
    ib.push_string("giraffe");
    assert!(!ib.all_is_read());
    let mut s = String::new();
    let n = ib.read_to_string(&mut s).unwrap();
    assert_eq!(n, 7);
    assert_eq!(s, "giraffe");
    assert!(ib.all_is_read());
    ib.clear();
    assert!(ib.all_is_read());
    let mut s2 = String::new();
    let n2 = ib.read_to_string(&mut s2).unwrap();
    assert_eq!(n2, 0);
    assert_eq!(s2, "");
    assert!(ib.all_is_read());
}

#[test]
fn output_buffer_write_empty() {
    let mut ob = OutputBuffer::new(10);
    write!(ob, "").unwrap();
    assert_eq!(ob.to_string().unwrap(), "");
}

#[test]
fn output_buffer_write_short() {
    let mut ob = OutputBuffer::new(10);
    write!(ob, "abc").unwrap();
    assert_eq!(ob.to_string().unwrap(), "abc");
}

#[test]
fn output_buffer_write_multiple() {
    let mut ob = OutputBuffer::new(10);
    write!(ob, "abc").unwrap();
    assert_eq!(ob.to_string().unwrap(), "abc");
    write!(ob, "abc").unwrap();
    assert_eq!(ob.to_string().unwrap(), "abcabc");
    write!(ob, "abc").unwrap();
    assert_eq!(ob.to_string().unwrap(), "abcabcabc");
}

#[test]
fn output_buffer_write_long() {
    let mut ob = OutputBuffer::new(10);
               //   0123456789
    write!(ob, "abcdefghijklmn").unwrap();
    assert_eq!(ob.to_string().unwrap(), "efghijklmn");
}

#[test]
fn output_buffer_clear() {
    let mut ob = OutputBuffer::new(10);
    write!(ob, "abc").unwrap();
    write!(ob, "0123456789").unwrap();
    assert_eq!(ob.to_string().unwrap(), "0123456789");
    ob.clear();
    assert_eq!(ob.to_string().unwrap(), "");
}

#[test]
fn output_buffer_read_before_write() {
    let ob = OutputBuffer::new(10);
    assert_eq!(ob.to_string().unwrap(), "");
}

#[test]
fn output_buffer_write_unicode() {
    let mut ob = OutputBuffer::new(5);
             // 4*1 + 1*2 = 6 bytes, 2-byte glyph on boundary
    write!(ob, "abcdé").unwrap();
                                      // 3*1 + 1*2 = 5 bytes
    assert_eq!(ob.to_string().unwrap(), "bcdé");
}

#[test]
fn output_buffer_invalid_unicode() {
    let mut ob = OutputBuffer::new(10);
    ob.write(&[60, 61, 130, 131]).unwrap();
    assert!(ob.to_string().is_err());
}

