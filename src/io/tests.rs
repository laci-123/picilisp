use pretty_assertions::assert_eq;
use std::io::{Write, Read};
use super::*;



#[test]
fn io_write_empty() {
    let (s, r) = mpsc::channel();
    let mut sender = IoSender::new(s);
    let mut receiver = IoReceiver::new(r, Duration::MAX);

    sender.write("".as_bytes()).unwrap();
    sender.flush().unwrap();

    let mut x = String::new();
    receiver.read_to_string(&mut x).unwrap();
    assert_eq!(x, "");
}

#[test]
fn io_write_short() {
    let (s, r) = mpsc::channel();
    let mut sender = IoSender::new(s);
    let mut receiver = IoReceiver::new(r, Duration::MAX);

    write!(sender, "abc").unwrap();
    sender.flush().unwrap();
    
    let mut x = String::new();
    receiver.read_to_string(&mut x).unwrap();
    assert_eq!(x, "abc");
}

#[test]
fn io_write_long() {
    let (s, r) = mpsc::channel();
    let mut sender = IoSender::new(s);
    let mut receiver = IoReceiver::new(r, Duration::MAX);

    write!(sender, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    sender.flush().unwrap();
    
    let mut x = String::new();
    receiver.read_to_string(&mut x).unwrap();
    assert_eq!(x, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
}

#[test]
fn io_write_multiple() {
    let (s, r) = mpsc::channel();
    let mut sender = IoSender::new(s);
    let mut receiver = IoReceiver::new(r, Duration::MAX);

    write!(sender, "abc").unwrap();
    write!(sender, "abc").unwrap();
    write!(sender, "abc").unwrap();
    sender.flush().unwrap();

    let mut x = String::new();
    receiver.read_to_string(&mut x).unwrap();
    assert_eq!(x, "abcabcabc");
}

#[test]
fn io_write_unicode() {
    let (s, r) = mpsc::channel();
    let mut sender = IoSender::new(s);
    let mut receiver = IoReceiver::new(r, Duration::MAX);

    write!(sender, "鯨は海の中で住んでいる。").unwrap();
    sender.flush().unwrap();
    
    let mut x = String::new();
    receiver.read_to_string(&mut x).unwrap();
    assert_eq!(x, "鯨は海の中で住んでいる。");
}

#[test]
fn io_invalid_unicode() {
    let (s, r) = mpsc::channel();
    let mut sender = IoSender::new(s);
    let mut receiver = IoReceiver::new(r, Duration::MAX);

    sender.write(&[91, 92, 93, 200, 201, 202, 203]).unwrap();
    sender.flush().unwrap();
    
    let mut x = String::new();
    assert!(receiver.read_to_string(&mut x).is_err());
}

#[test]
fn io_timeout() {
    let (_s, r) = mpsc::channel();
    let mut receiver = IoReceiver::new(r, Duration::from_millis(1));

    let mut x = String::new();
    let err = receiver.read_to_string(&mut x);
    assert_eq!(err.err().unwrap().kind(), io::ErrorKind::TimedOut);
}

