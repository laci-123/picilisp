use pretty_assertions::assert_eq;
use std::io::{Read, Write};
use super::*;



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

