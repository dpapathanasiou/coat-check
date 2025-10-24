use coat_check::server::Server;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::{thread, time};

mod common;

fn test_harness(n: i32, actions: Vec<String>, expectations: Vec<String>) {
    let server = Server {
        port: 5000 + n as u16,
        filepath: common::generate_test_file(n),
    };
    // server start() never returns, so spin it up in the background
    thread::spawn(move || {
        server.start().unwrap();
    });

    // pause long enough to have the server start accepting connections
    thread::sleep(time::Duration::from_millis(100));

    // run the tests as the client
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", 5000 + n)).unwrap();

    let mut buf = [0u8; 1024];
    let mut read_size: usize;
    let mut l: usize;

    for (act, exp) in actions.iter().zip(expectations.iter()) {
        // write the action
        l = act.len();
        buf[0..l].copy_from_slice(act.as_bytes());
        buf[l..l + 2].copy_from_slice(b"\r\n");
        stream.write_all(&buf).unwrap();

        // read the server reply
        stream.read(&mut buf).unwrap();
        read_size = buf
            .clone()
            .iter()
            .take_while(|c| **c != b'\n' && **c != b'\r')
            .count();
        assert_eq!(str::from_utf8(&buf[0..read_size]).unwrap(), exp);
    }

    // send the telnet quit command
    buf.fill(0);
    let quit = b"\x1D\r\nquit\r\n";
    let q = quit.len();
    buf[0..q].copy_from_slice(quit);
    stream.write_all(&buf).unwrap();
}

#[test]
fn server_write_then_read_key_works() {
    let actions = ["set foo my value", "get foo"];
    let expectations = ["*** success: wrote 49 bytes", "my value"];

    test_harness(
        1,
        actions.iter().map(|&s| s.into()).collect(),
        expectations.iter().map(|&s| s.into()).collect(),
    );
}

#[test]
fn server_duplicate_key_writes_do_not_upsert() {
    let actions = ["set foo my value", "set foo 한국어 키보드", "get foo"];
    let expectations = [
        "*** success: wrote 49 bytes",
        "*** success: wrote 0 bytes",
        "my value",
    ];

    test_harness(
        2,
        actions.iter().map(|&s| s.into()).collect(),
        expectations.iter().map(|&s| s.into()).collect(),
    );
}

#[test]
fn server_unknown_key_no_match() {
    let actions = ["set foo my value", "get foobar"];
    let expectations = ["*** success: wrote 49 bytes", "*** no match found"];

    test_harness(
        3,
        actions.iter().map(|&s| s.into()).collect(),
        expectations.iter().map(|&s| s.into()).collect(),
    );
}

#[test]
fn server_invalid_command_warning() {
    let actions = ["what is your name?"];
    let expectations = ["*** invalid command"];

    test_harness(
        4,
        actions.iter().map(|&s| s.into()).collect(),
        expectations.iter().map(|&s| s.into()).collect(),
    );
}
