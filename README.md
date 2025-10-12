# About

This [key-value store](https://en.wikipedia.org/wiki/Key%E2%80%93value_database) service is the capstone project for [CS644: Intro to Systems Programming](https://iafisher.com/cs644), [Fall 2025](https://iafisher.com/cs644/fall2025)

The name [coat check](https://dictionary.cambridge.org/example/english/coat-check) is a fanciful real-world analogy to what this software does with data

# Design

Given the requirement to store all key-value pairs in a single file, this is a first attempt at a data format and algorithm that can accommodate variable-sized values:

```
[key] - hashed, fixed length
[size of value] - in bytes, usize
[value] - variable length
...
```

- Fetches work by reading the first n bytes of the key, and if equivalent, returning the corresponding value; otherwise, the size parameter found just after the key is used to skip (`lseek`) ahead to the next key, and the process repeats until either a match is found, or end of file is reached
- Inserts work by confirming the key does not already exist, and if so, adding the `[key][size of value][value]` bytes to the end of the file
- Attempting to write the same key more than once does *not* result in an [upsert](https://en.wikipedia.org/wiki/Merge_%28SQL%29), the original value remains

## Limitations

While the data format meets the basic requirements, including the ability to accommodate a value of any size and type, it also has the following limitations:

- Does not scale easily, since writes are accepted in the order received, and reads do not have the benefit of using an index, etc.
- Keys must hash to the same size, otherwise the read algorithm does not work
- No way to delete or upsert existing key-value pairs

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

Clients use [telnet]() to interact with the server and data:

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
$ cargo test -- --no-capture
   Compiling coat-check v0.1.0 (...)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.41s
     Running unittests src/lib.rs (target/debug/deps/coat_check-da541ad98bad8f52)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/coat_check-49960524fe20ac81)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/file_syscall_tests.rs (target/debug/deps/file_syscall_tests-fccbd6767bc74354)

running 4 tests
test first_read_key_fails ... ok
test duplicate_key_writes_do_not_upsert ... ok
test write_then_read_key_works ... ok
test lock_on_writes_blocks_reads_without_errors ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s

     Running tests/hasher_test.rs (target/debug/deps/hasher_test-9d0fbf42de0cbf4c)

running 2 tests
test key_hashing_is_deterministic ... ok
test key_hashing_is_case_sensitive ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests coat_check

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
