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
- Deletes and upserts waste space until [compaction is requested](#compacting-the-data-file) explicitly

# Usage

This is a typical [Rust](https://www.rust-lang.org/) application, using [Cargo](https://doc.rust-lang.org/cargo/index.html), so all the [normal commands](https://doc.rust-lang.org/cargo/commands/index.html) work as expected:

## Run

### Transactional: 'get', 'set', or 'del' one at a time

```sh
$ cargo run set foo "this is the value for 'foo'"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/coat-check set foo 'this is the value for '\''foo'\'''`
fork(wc): parent pid 42513 -> child pid 42519
fork(wc): in child -> pid 42519
wc: /tmp/data.coat-check: No such file or directory
fork(wc): in parent -> child pid 42519 exited, status = 1
[2025-10-26T18:00:33Z INFO  coat_check] success: wrote 68 bytes
fork(wc): parent pid 42513 -> child pid 42520
fork(wc): in child -> pid 42520
68 /tmp/data.coat-check
fork(wc): in parent -> child pid 42520 exited, status = 0
```

```sh
$ cargo run get foo
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/coat-check get foo`
fork(wc): parent pid 42527 -> child pid 42533
fork(wc): in child -> pid 42533
68 /tmp/data.coat-check
fork(wc): in parent -> child pid 42533 exited, status = 0
[2025-10-26T18:01:08Z INFO  coat_check] success: matched -> Ok("this is the value for 'foo'")
fork(wc): parent pid 42527 -> child pid 42534
fork(wc): in child -> pid 42534
68 /tmp/data.coat-check
fork(wc): in parent -> child pid 42534 exited, status = 0
```

```sh
$ cargo run del foo
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/coat-check del foo`
fork(wc): parent pid 42606 -> child pid 42612
fork(wc): in child -> pid 42612
68 /tmp/data.coat-check
fork(wc): in parent -> child pid 42612 exited, status = 0
[2025-10-26T18:02:07Z INFO  coat_check] success: deleted value -> Ok("this is the value for 'foo'")
fork(wc): parent pid 42606 -> child pid 42613
fork(wc): in child -> pid 42613
68 /tmp/data.coat-check
fork(wc): in parent -> child pid 42613 exited, status = 0

$ cargo run get foo
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/coat-check get foo`
fork(wc): parent pid 42669 -> child pid 42675
fork(wc): in child -> pid 42675
68 /tmp/data.coat-check
fork(wc): in parent -> child pid 42675 exited, status = 0
[2025-10-26T18:02:39Z INFO  coat_check] no match found
fork(wc): parent pid 42669 -> child pid 42676
fork(wc): in child -> pid 42676
68 /tmp/data.coat-check
fork(wc): in parent -> child pid 42676 exited, status = 0
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
*** success: wrote 71 bytes
get bar
私は毎日勉強します。
del bar
私は毎日勉強します。
get bar
*** no match found
what?
*** invalid command
Usage:
<get> <key> | <set> <key> <value> | <del> <key>
^]
telnet> close
Connection closed.
```

### Compacting the data file

This command removes all records whose deleted flag is true from the data file:

```sh
$ cargo run compact
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/coat-check compact`
     ...
[2025-11-01T14:58:08Z INFO  coat_check] compact complete
```

While running in server mode, it is also possible to send a [signal](https://www.man7.org/linux/man-pages/man7/signal.7.html) of type `SIGUSR2` which sets the compaction to happen at the next client connection, before it spawns a new thread to handle the new connection:

```sh
(client) $ kill -12 [pid]
...
(server) [running as `pid`]
    Server listening on 5000 -> 3
    Connected to client: 4 -> "/tmp/data.coat-check"
    sig :: Received compact signal 12!
    sig :: COMPACT_SIGNALED contains true
    Compacting "/tmp/data.coat-check" -- please wait
    Compacting "/tmp/data.coat-check" -- completed
```




## Test
```sh
$ cargo test
...
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    Running unittests src/main.rs (target/debug/deps/coat_check-d653779de2de4076)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    Running tests/common.rs (target/debug/deps/common-e18dd3a678e2d924)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    Running tests/file_syscall_tests.rs (target/debug/deps/file_syscall_tests-1bc8364e6a863a5d)

running 7 tests
test first_read_key_fails ... ok
test duplicate_key_writes_upsert ... ok
test write_then_delete_key_works ... ok
test write_then_read_key_works ... ok
test write_then_delete_key_multiple_time_produces_last_value ... ok
test compaction_works ... ok
test lock_on_writes_blocks_reads_without_errors ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s

    Running tests/hasher_test.rs (target/debug/deps/hasher_test-48d45347db5a6769)

running 2 tests
test key_hashing_is_deterministic ... ok
test key_hashing_is_case_sensitive ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

    Running tests/server_tests.rs (target/debug/deps/server_tests-3d82c51ee34446ef)

running 5 tests
Connected to client: 4 -> "/tmp/test-1762008123-4.coat-check"
Connected to client: 6 -> "/tmp/test-1762008123-5.coat-check"
Connected to client: 10 -> "/tmp/test-1762008123-2.coat-check"
test server_invalid_command_warning ... ok
Disconnected from client: 6 -> "/tmp/test-1762008123-5.coat-check"
Disconnected from client: 4 -> "/tmp/test-1762008123-4.coat-check"
test server_delete_key_works ... ok
test server_duplicate_key_writes_upsert ... ok
Connected to client: 8 -> "/tmp/test-1762008123-3.coat-check"
Disconnected from client: 10 -> "/tmp/test-1762008123-2.coat-check"
test server_unknown_key_no_match ... Disconnected from client: 8 -> "/tmp/test-1762008123-3.coat-check"
ok
Connected to client: 12 -> "/tmp/test-1762008123-1.coat-check"
test server_write_then_read_key_works ... ok
Disconnected from client: 12 -> "/tmp/test-1762008123-1.coat-check"

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

  Doc-tests coat_check

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
