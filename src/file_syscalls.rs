use crate::hasher;
use nix::errno::Errno;
use nix::fcntl::{Flock, FlockArg, OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, lseek, read, write};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};

const SPACER: usize = std::mem::size_of::<usize>();

fn record_reader<F>(
    fd: &BorrowedFd,
    key: &str,
    val: &[u8],
    matchop: F,
) -> Result<Option<Vec<u8>>, Errno>
where
    F: Fn(&BorrowedFd, [u8; SPACER], &str, &[u8]) -> Result<Option<Vec<u8>>, Errno>,
{
    let size_buf: &mut [u8] = &mut vec![0; SPACER];
    let mut sizer: [u8; SPACER] = [0; SPACER];

    let hash = hasher::hash_key(key);
    let key_buf: &mut [u8] = &mut vec![0; hash.len()];
    let del_buf: &mut [u8] = &mut vec![0; 1];

    // iterate through the records (`[(hashed) key][size of value][deleted?][value]` byte arrays) in file
    let mut nbytes = read(fd, key_buf)?;
    while nbytes > 0 {
        if hash != str::from_utf8(key_buf).unwrap() {
            // no match at the current key position,
            // so read the value size, and skip ahead
            // to the next key/value array
            _ = read(fd, size_buf)?;
            sizer.clone_from_slice(size_buf);
            _ = read(fd, del_buf)?; // skip over the delete flag
            lseek(fd, i64::from_ne_bytes(sizer), Whence::SeekCur)?;
        } else {
            // matched, so get the value size, and execute the matchop function
            _ = read(fd, size_buf)?;
            sizer.clone_from_slice(size_buf);

            match matchop(fd, sizer, key, val) {
                Ok(result) => match result {
                    Some(data) => return Ok(Some(data)), // stop iterating through the file
                    None => (),
                },
                Err(e) => return Err(e),
            }
        }
        nbytes = read(fd, key_buf)?;
    }

    // reached the EOF without a match: use EKEYEXPIRED (Key has expired) as the return value
    Err(Errno::EKEYEXPIRED)
}

/* Higher order functions, to use as `matchop: F` in `record_reader<F>`
 *
 * find()   used by read_key()
 * delete() used by delete_key()
 *
 */

fn find(fd: &BorrowedFd, sizer: [u8; SPACER], _: &str, _: &[u8]) -> Result<Option<Vec<u8>>, Errno> {
    // read the deleted flag
    let del_buf: &mut [u8] = &mut vec![0; 1];
    _ = read(fd, del_buf)?;
    // also read the value, regardless, otherwise the record_reader() loop will not continue in the correct position
    let val_buf: &mut [u8] = &mut vec![0; usize::from_ne_bytes(sizer)];
    _ = read(fd, val_buf)?;
    if del_buf[0] == 0 {
        // not deleted, so return the corresponding value as a match
        let mut result = Vec::new();
        result.extend_from_slice(val_buf);
        return Ok(Some(result));
    }
    // not found on this iteration of the record_reader() loop
    Ok(None)
}

fn delete(
    fd: &BorrowedFd,
    sizer: [u8; SPACER],
    _: &str,
    _: &[u8],
) -> Result<Option<Vec<u8>>, Errno> {
    // record the current file position, before reading the deleted flag
    let current_pos = lseek(fd, 0, Whence::SeekCur)?;
    // read the deleted flag
    let del_buf: &mut [u8] = &mut vec![0; 1];
    _ = read(fd, del_buf)?;
    // also read the value, regardless, otherwise the record_reader() loop will not continue in the correct position
    let val_buf: &mut [u8] = &mut vec![0; usize::from_ne_bytes(sizer)];
    _ = read(fd, val_buf)?;
    if del_buf[0] == 0 {
        // not deleted, so back up, and overwrite the deleted flag to true
        lseek(fd, current_pos, Whence::SeekSet)?;
        let deleted: &mut [u8] = &mut vec![1; 1];
        _ = write(fd, deleted)?;

        // return the corresponding value, so that the caller knows to stop iterating
        let mut result = Vec::new();
        result.extend_from_slice(val_buf);
        return Ok(Some(result));
    }
    // not found on this iteration of the record_reader() loop
    Ok(None)
}

/* Public API  */

pub fn read_key(filepath: String, key: &str) -> Result<Option<Vec<u8>>, Errno> {
    let fd: OwnedFd = open(filepath.as_str(), OFlag::O_RDONLY, Mode::empty())?;
    let lock = match Flock::lock(fd, FlockArg::LockShared) {
        Ok(locked) => locked,
        Err((_, e)) => return Err(e),
    };

    let mut result: Result<Option<Vec<u8>>, Errno>;
    loop {
        let empty_buf: &mut [u8] = &mut vec![0; 1];
        result = record_reader(&lock.as_fd(), key, empty_buf, find);
        // stop if found a matching, non-deleted key
        if result.is_ok() {
            break;
        }
        // or if reached EOF
        if result.is_err_and(|x| x == Errno::EKEYEXPIRED) {
            result = Ok(None);
            break;
        }
    }

    match lock.unlock() {
        Ok(unlocked) => {
            close(unlocked)?;
            result
        }
        Err((_, e)) => Err(e),
    }
}

pub fn delete_key(filepath: String, key: &str) -> Result<Option<Vec<u8>>, Errno> {
    let fd: OwnedFd = open(
        filepath.as_str(),
        OFlag::O_RDWR,
        Mode::S_IRUSR
            | Mode::S_IWUSR
            | Mode::S_IRGRP
            | Mode::S_IWGRP
            | Mode::S_IROTH
            | Mode::S_IWOTH,
    )?;
    let lock = match Flock::lock(fd, FlockArg::LockExclusive) {
        Ok(locked) => locked,
        Err((_, e)) => return Err(e),
    };

    let mut result: Result<Option<Vec<u8>>, Errno>;
    loop {
        let empty_buf: &mut [u8] = &mut vec![0; 1];
        result = record_reader(&lock.as_fd(), key, empty_buf, delete);
        // stop if found a matching key which was previously non-deleted
        if result.is_ok() {
            break;
        }
        // or if reached EOF
        if result.is_err_and(|x| x == Errno::EKEYEXPIRED) {
            result = Ok(None);
            break;
        }
    }

    match lock.unlock() {
        Ok(unlocked) => {
            close(unlocked)?;
            result
        }
        Err((_, e)) => Err(e),
    }
}

fn append_new_key_val(filepath: String, key: &str, val: &[u8]) -> Result<usize, Errno> {
    let fd: OwnedFd = open(
        filepath.as_str(),
        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_APPEND,
        Mode::S_IRUSR
            | Mode::S_IWUSR
            | Mode::S_IRGRP
            | Mode::S_IWGRP
            | Mode::S_IROTH
            | Mode::S_IWOTH,
    )?;
    let lock = match Flock::lock(fd, FlockArg::LockExclusive) {
        Ok(locked) => locked,
        Err((_, e)) => return Err(e),
    };

    // produce a new record (`[(hashed) key][size of value][deleted?][value]` byte array), given the key and value data
    let hash = hasher::hash_key(key);
    let hash_size = hash.len();
    let val_size = val.iter().count();
    let sizer: [u8; SPACER] = val_size.to_ne_bytes();
    let sizer_size = sizer.len();
    let deleted: [u8; 1] = [0];
    let deleted_size = deleted.len();
    let buffer: &mut [u8] = &mut vec![0; hash_size + sizer_size + deleted_size + val_size];
    buffer[0..hash_size].copy_from_slice(hash.as_bytes());
    buffer[hash_size..hash_size + sizer_size].copy_from_slice(&sizer);
    buffer[hash_size + sizer_size..hash_size + sizer_size + deleted_size].copy_from_slice(&deleted);
    buffer[hash_size + sizer_size + deleted_size..].copy_from_slice(val);

    // append it to the end of the file
    let nbytes = write(lock.as_fd(), buffer)?;

    match lock.unlock() {
        Ok(unlocked) => {
            close(unlocked)?;
            Ok(nbytes)
        }
        Err((_, e)) => Err(e),
    }
}

pub fn write_key_val(filepath: String, key: &str, val: &[u8]) -> Result<usize, Errno> {
    // before writing this as a new key-value pair, make sure it does not already exist
    match read_key(filepath.clone(), key) {
        Ok(result) => match result {
            Some(value_vector) => {
                // key already exists, so upsert if the value is different
                if val == value_vector {
                    Ok(0)
                } else {
                    // upsert: delete the current key, and append the new value
                    match delete_key(filepath.clone(), key) {
                        Ok(_) => append_new_key_val(filepath, key, val),
                        Err(e) => Err(e),
                    }
                }
            }
            None => append_new_key_val(filepath, key, val),
        },
        Err(e) => match e {
            Errno::ENOENT => append_new_key_val(filepath, key, val), // file does not exist yet, so create it with this as the first entry
            _ => Err(e),
        },
    }
}
