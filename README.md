# About

This [key-value store](https://en.wikipedia.org/wiki/Key%E2%80%93value_database) service is the capstone project for [CS644: Intro to Systems Programming](https://iafisher.com/cs644), [Fall 2025](https://iafisher.com/cs644/fall2025)

The name [coat check](https://dictionary.cambridge.org/example/english/coat-check) is a fanciful real-world analogy to what this software does with data

# Design

Given the requirement to store all key-value records in a single file, this is a data format and algorithm that can accommodate values of different sizes:

![Data Format Diagram](doc/data.png?raw=true)

- Fetches work by reading the first *n* bytes of the key, and if equivalent, returning the corresponding value when the deleted flag is false; otherwise, the size parameter found just after the key is used to skip (`lseek`) ahead to the next key, and the process repeats until either a match is found, or end of file is reached
- Inserts work by confirming the key does not already exist without the deleted flag set to true, and if so, adds the new record (`[key][size of value][deleted?][value]` bytes) to the end of the file
- Attempting to write the same key more than once results in an [upsert](https://en.wikipedia.org/wiki/Merge_%28SQL%29): the original value gets its deleted flag set to true, and a new record, using the new value, gets written as a new record to the end of the file

## Limitations

While the data format meets the basic requirements, including the ability to accommodate a value of any size and type, it also has the following limitations:

- Does not scale easily, since writes are accepted in the order received, and reads do not have the benefit of using an index, etc.
- Keys must hash to the same size, otherwise the read algorithm does not work
- Deletes and upserts waste space

# Usage

This is a typical [Rust](https://www.rust-lang.org/) application, using [Cargo](https://doc.rust-lang.org/cargo/index.html), so all the [normal commands](https://doc.rust-lang.org/cargo/commands/index.html) work as expected:

## Run

### Transactional: 'get' or 'set' one at a time

```sh
$ cargo run set foo "this is the value for 'foo'"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
    ...
[2025-10-12T16:15:45Z INFO  coat_check] success: wrote 67 bytes
fork(wc): parent pid 22898 -> child pid 22905
fork(wc): in child -> pid 22905
67 /tmp/data.coat-check
fork(wc): in parent -> child pid 22905 exited, status = 0
```

```sh
$ cargo run get foo
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
    ...
[2025-10-12T16:15:54Z INFO  coat_check] success: matched -> Ok("this is the value for 'foo'")
fork(wc): parent pid 22906 -> child pid 22913
fork(wc): in child -> pid 22913
67 /tmp/data.coat-check
fork(wc): in parent -> child pid 22913 exited, status = 0
```

### Server Mode

```sh
$ cargo run server
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
    ...
Server listening on 5000
```

Clients use [telnet](https://en.wikipedia.org/wiki/Telnet) to interact with the server and data:

```sh
$ telnet localhost 5000
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
get foo
this is the value for 'foo'
get bar
*** no match found
set bar 私は毎日勉強します。
*** success: wrote 70 bytes
get bar
私は毎日勉強します。
what?
*** invalid command
Usage:
<get> <key> | <set> <key> <value>
^]
telnet> close
Connection closed.
```

## Test
```sh
$ cargo test
   Compiling coat-check v0.1.0 (/home/denis/repos/repos-git/coat-check)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/lib.rs (target/debug/deps/coat_check-d0433ddf8bcd52a9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/coat_check-e5512602a7a735ed)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/common.rs (target/debug/deps/common-747e04b5efe1f5b4)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/file_syscall_tests.rs (target/debug/deps/file_syscall_tests-27fc31fa7f20d9b0)

running 4 tests
test first_read_key_fails ... ok
test duplicate_key_writes_do_not_upsert ... ok
test write_then_read_key_works ... ok
test lock_on_writes_blocks_reads_without_errors ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s

     Running tests/hasher_test.rs (target/debug/deps/hasher_test-4267c0a303a982e7)

running 2 tests
test key_hashing_is_deterministic ... ok
test key_hashing_is_case_sensitive ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/server_tests.rs (target/debug/deps/server_tests-9a988a8d33482924)

running 4 tests
test server_invalid_command_warning ... ok
test server_write_then_read_key_works ... ok
test server_duplicate_key_writes_do_not_upsert ... ok
test server_unknown_key_no_match ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests coat_check

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
