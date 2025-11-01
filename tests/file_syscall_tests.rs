use coat_check::file_syscalls::{compact, delete_key, read_key, write_key_val};
use nix::errno::Errno;
use std::thread;
use std::time::Duration;

mod common;

#[test]
fn first_read_key_fails() {
    let file_folder = common::generate_test_file(0);

    let result = read_key(file_folder, "meh");
    // file does not exist, so read() should fail
    assert!(result.is_err());
    assert_eq!(result, Err(Errno::ENOENT))
}

#[test]
fn write_then_read_key_works() {
    let file_folder = common::generate_test_file(1);

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
fn write_then_delete_key_works() {
    let file_folder = common::generate_test_file(2);

    // write() to a non-existent file should succeed
    let test_key = "boo";
    let expected = b"some value goes here";
    let write_result = write_key_val(file_folder.clone(), test_key, expected);
    assert!(write_result.is_ok());

    // attempt to delete the just-written key
    let delete_result = delete_key(file_folder.clone(), test_key);
    match delete_result {
        Ok(bytes) => match bytes {
            Some(value_vector) => assert_eq!(value_vector, expected), // successful delete returns the corresponding value, for reference
            None => assert!(false),
        },
        Err(_) => assert!(false),
    }

    // reading back the just-deleted key should fail
    let read_result = read_key(file_folder.clone(), test_key);
    match read_result {
        Ok(bytes) => match bytes {
            Some(_) => assert!(false),
            None => assert!(true),
        },
        Err(_) => assert!(false),
    }
}

#[test]
fn write_then_delete_key_multiple_time_produces_last_value() {
    let file_folder = common::generate_test_file(3);

    // same key but different values
    let key = "katakana";
    let vals = vec!["あ", "い", "う", "え", "お"];
    let cases = vals.len();

    for i in 0..cases {
        // write the new value for the key
        let write_result = write_key_val(file_folder.clone(), key, vals.get(i).unwrap().as_bytes());
        assert!(write_result.is_ok());

        // reading back the just-written key should work
        let read_result = read_key(file_folder.clone(), key);
        match read_result {
            Ok(bytes) => match bytes {
                Some(_) => assert!(true),
                None => assert!(false),
            },
            Err(_) => assert!(false),
        }

        if i != cases - 1 {
            // delete the key before attempting to write the next value
            let delete_result = delete_key(file_folder.clone(), key);
            assert!(delete_result.is_ok());
        }
    }

    // final read of the key should be the last value
    let read_result = read_key(file_folder.clone(), key);
    let expected = vals.last().unwrap().as_bytes();
    match read_result {
        Ok(bytes) => match bytes {
            Some(value_vector) => assert_eq!(value_vector, expected),
            None => assert!(false),
        },
        Err(_) => assert!(false),
    }
}

#[test]
fn duplicate_key_writes_upsert() {
    let file_folder = common::generate_test_file(4);

    // write() to a non-existent file should succeed
    let test_key = "the key";
    let first_val = b"this is the first value for the key";
    let first_write_result = write_key_val(file_folder.clone(), test_key, first_val);
    assert!(first_write_result.is_ok());

    let second_val = b"a different value for the same key";
    let second_write_result = write_key_val(file_folder.clone(), test_key, second_val);
    assert!(second_write_result.is_ok());
    match second_write_result {
        Ok(bytes) => assert_ne!(bytes, 0), // i.e., it did write the new value
        Err(_) => assert!(false),
    }

    // and reading back the just-written key should succeed and match the second value
    let read_result = read_key(file_folder.clone(), test_key);
    match read_result {
        Ok(bytes) => match bytes {
            Some(value_vector) => assert_eq!(value_vector, second_val),
            None => assert!(false),
        },
        Err(_) => assert!(false),
    }

    // but attempting to rewrite the same key-value pair should result in no action
    let second_write_result_redux = write_key_val(file_folder.clone(), test_key, second_val);
    assert!(second_write_result_redux.is_ok());
    match second_write_result_redux {
        Ok(bytes) => assert_eq!(bytes, 0), // i.e., it did not write the new value, since it was the same as before
        Err(_) => assert!(false),
    }
}

#[test]
fn lock_on_writes_blocks_reads_without_errors() {
    let file_folder = common::generate_test_file(5);

    let keys = vec![
        "α", "β", "γ", "δ", "ε", "ζ", "η", "θ", "ι", "κ", "λ", "μ", "ν", "ξ", "ο", "π", "ρ", "σ",
        "τ", "υ", "φ", "χ", "ψ", "ω",
    ];
    let vals = vec![
        "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota", "Kappa",
        "Lambda", "Mu", "Nu", "Xi", "Omicron", "Pi", "Rho", "Sigma", "Tau", "Upsilon", "Phi",
        "Chi", "Psi", "Omega",
    ];

    // write the first key-value pair to the file so that all the subsequent reads in the main thread work
    let first_write_result = write_key_val(
        file_folder.clone(),
        keys.get(0).unwrap(),
        vals.get(0).unwrap().as_bytes(),
    );
    assert!(first_write_result.is_ok());

    // prepare for the remaining key-value writes as spawned threads (borrow all these, b/c of the upcoming 'move')
    let f = file_folder.clone();
    let k = keys.clone();
    let v = vals.clone();
    let cases = keys.len();

    thread::spawn(move || {
        for i in 1..cases {
            match write_key_val(f.clone(), k.get(i).unwrap(), v.get(i).unwrap().as_bytes()) {
                Ok(bytes) => assert_ne!(bytes, 0), // as brand-new writes, these should all be > 0
                Err(_) => assert!(false),
            }
            thread::sleep(Duration::from_millis(50));
        }
    });

    // main thread: make sure to confirm all the keys, waiting on the writer threads to finish and release their locks
    for i in 0..cases {
        let key = keys.get(i).unwrap();
        let val = vals.get(i).unwrap().as_bytes();
        let mut matched = false;
        while !matched {
            let read_result = read_key(file_folder.clone(), key);
            assert!(read_result.is_ok());
            matched = match read_result {
                Ok(bytes) => match bytes {
                    Some(value_vector) => value_vector == val,
                    None => false, // there may not be a match yet
                },
                Err(_) => false,
            };
        }
    }
}

#[test]
fn compaction_works() {
    let file_folder = common::generate_test_file(6);

    let keys = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
    let vals = vec![
        "uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez",
    ];

    let cases = keys.len();
    for i in 0..cases {
        let k = keys.get(i).unwrap();
        let v = vals.get(i).unwrap().as_bytes();
        match write_key_val(file_folder.clone(), k, v) {
            Ok(bytes) => assert_ne!(bytes, 0), // as brand-new writes, these should all be > 0
            Err(_) => assert!(false),
        }
        if i % 2 == 0 {
            // delete the odd half, so that test file needs compaction
            match delete_key(file_folder.clone(), k) {
                Ok(bytes) => match bytes {
                    Some(value_vector) => assert_eq!(value_vector, v), // successful delete returns the corresponding value, for reference
                    None => assert!(false),
                },
                Err(_) => assert!(false),
            }
        }
    }

    // compact the test file
    match compact(file_folder.clone()) {
        Ok(bytes) => match bytes {
            Some(_) => assert!(false),
            None => assert!(true),
        },
        Err(_) => assert!(false),
    }

    // iterate through keys/vals and confirm only the even ones exist
    for i in 0..cases {
        let k = keys.get(i).unwrap();
        let read_result = read_key(file_folder.clone(), k);
        assert!(read_result.is_ok());
        if i % 2 == 0 {
            // expect these to be missing
            match read_result {
                Ok(bytes) => match bytes {
                    Some(_) => assert!(false),
                    None => assert!(true),
                },
                Err(_) => assert!(false),
            };
        } else {
            // expect these to exist
            match read_result {
                Ok(bytes) => match bytes {
                    Some(value_vector) => assert_eq!(value_vector, vals.get(i).unwrap().as_bytes()),
                    None => assert!(false),
                },
                Err(_) => assert!(false),
            };
        }
    }
}
