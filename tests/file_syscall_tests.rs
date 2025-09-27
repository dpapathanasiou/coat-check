use chrono::Utc;
use coat_check::file_syscalls::{read_key, write_key_val};
use nix::errno::Errno;

fn generate_test_file(n: i32) -> String {
    format!(
        "/tmp/test-{}-{n}.coat-check",
        Utc::now().timestamp().to_string()
    )
}

#[test]
fn first_read_key_fails() {
    let file_folder = generate_test_file(0);

    let result = read_key(file_folder, "meh");
    // file does not exist, so read() should fail
    assert!(result.is_err());
    assert_eq!(result, Err(Errno::ENOENT))
}

#[test]
fn write_then_read_key_works() {
    let file_folder = generate_test_file(1);

    // write() to a non-existent file should succeed
    let test_key = "boo";
    let expected = b"some value goes here";
    let write_result = write_key_val(file_folder.clone(), test_key, expected);
    assert!(write_result.is_ok());

    // and reading back the just-written key should succeed and match
    let read_result = read_key(file_folder.clone(), test_key);
    match read_result {
        Ok(bytes) => match bytes {
            Some(value_vector) => assert_eq!(value_vector, expected),
            None => assert!(false),
        },
        Err(_) => assert!(false),
    }
}

#[test]
fn duplicate_key_writes_do_not_upsert() {
    let file_folder = generate_test_file(2);

    // write() to a non-existent file should succeed
    let test_key = "the key";
    let first_val = b"this is the first value for the key";
    let first_write_result = write_key_val(file_folder.clone(), test_key, first_val);
    assert!(first_write_result.is_ok());

    let second_val = b"a different value for the same key";
    let second_write_result = write_key_val(file_folder.clone(), test_key, second_val);
    assert!(second_write_result.is_ok());
    match second_write_result {
        Ok(bytes) => assert_eq!(bytes, 0), // i.e., it did not write the new value
        Err(_) => assert!(false),
    }

    // and reading back the just-written key should succeed and match
    let read_result = read_key(file_folder.clone(), test_key);
    match read_result {
        Ok(bytes) => match bytes {
            Some(value_vector) => assert_eq!(value_vector, first_val),
            None => assert!(false),
        },
        Err(_) => assert!(false),
    }
}
