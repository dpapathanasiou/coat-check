use crate::hasher;
use nix::errno::Errno;
use nix::fcntl::{Flock, FlockArg, OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, lseek, read, write};
use std::os::fd::{AsFd, OwnedFd};

pub fn read_key(filepath: String, key: &str) -> Result<Option<Vec<u8>>, Errno> {
    let fd: OwnedFd = open(filepath.as_str(), OFlag::O_RDONLY, Mode::empty())?;
    let lock = match Flock::lock(fd, FlockArg::LockShared) {
        Ok(locked) => locked,
        Err((_, e)) => return Err(e),
    };

    const SPACER: usize = std::mem::size_of::<usize>();
    let size_buf: &mut [u8] = &mut vec![0; SPACER];
    let mut sizer: [u8; SPACER] = [0; SPACER];

    let hash = hasher::hash_key(key);
    let key_buf: &mut [u8] = &mut vec![0; hash.len()];

    // iterate through the `[(hashed) key][size of value][value]` byte arrays in file
    let mut nbytes = read(lock.as_fd(), key_buf)?;
    while nbytes > 0 {
        if hash != str::from_utf8(key_buf).unwrap() {
            // no match at the current key position,
            // so read the value size, and skip ahead
            // to the next key/value array
            _ = read(lock.as_fd(), size_buf)?;
            sizer.clone_from_slice(size_buf);
            lseek(lock.as_fd(), i64::from_ne_bytes(sizer), Whence::SeekCur)?;
            nbytes = read(lock.as_fd(), key_buf)?;
        } else {
            // matched, so get the value size, to read and return the value bytes
            _ = read(lock.as_fd(), size_buf)?;
            sizer.clone_from_slice(size_buf);
            let val_buf: &mut [u8] = &mut vec![0; usize::from_ne_bytes(sizer)];
            _ = read(lock.as_fd(), val_buf)?;
            let mut result = Vec::new();
            result.extend_from_slice(val_buf);
            drop(lock);
            return Ok(Some(result));
        }
    }

    drop(lock);
    Ok(None)
}

fn write_new_key_val(filepath: String, key: &str, val: &[u8]) -> Result<usize, Errno> {
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

    // produce a `[(hashed) key][size of value][value]` byte array, given the key and value data
    let hash = hasher::hash_key(key);
    let hash_size = hash.len();
    let val_size = val.iter().count();
    let sizer: [u8; std::mem::size_of::<usize>()] = val_size.to_ne_bytes();
    let sizer_size = sizer.len();
    let buffer: &mut [u8] = &mut vec![0; hash_size + sizer_size + val_size];
    buffer[0..hash_size].copy_from_slice(hash.as_bytes());
    buffer[hash_size..hash_size + sizer_size].copy_from_slice(&sizer);
    buffer[hash_size + sizer_size..].copy_from_slice(val);

    let nbytes = write(lock.as_fd(), buffer)?;
    drop(lock);
    Ok(nbytes)
}

pub fn write_key_val(filepath: String, key: &str, val: &[u8]) -> Result<usize, Errno> {
    // before writing this as a new key-value pair, make sure it does not already exist
    match read_key(filepath.clone(), key) {
        Ok(result) => match result {
            Some(_) => Ok(0), // key already exists, so do nothing
            None => write_new_key_val(filepath, key, val),
        },
        Err(e) => match e {
            Errno::ENOENT => write_new_key_val(filepath, key, val), // file does not exist yet, so create it with this as the first entry
            _ => Err(e),
        },
    }
}
