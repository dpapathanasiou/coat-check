use crate::hasher;
use nix::errno::Errno;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{Whence, close, lseek, read, write};
use std::os::unix::io::OwnedFd;

pub fn read_key(filepath: String, key: &str) -> Result<Option<Vec<u8>>, Errno> {
    let fd: OwnedFd = open(filepath.as_str(), OFlag::O_RDONLY, Mode::empty())?;

    const SPACER: usize = std::mem::size_of::<usize>();
    let size_buf: &mut [u8] = &mut vec![0; SPACER];
    let mut sizer: [u8; SPACER] = [0; SPACER];

    let hash = hasher::hash_key(key);
    let key_buf: &mut [u8] = &mut vec![0; hash.len()];

    // iterate through the `[(hashed) key][size of value][value]` byte arrays in file
    let mut nbytes = read(&fd, key_buf)?;
    while nbytes > 0 {
        if hash != str::from_utf8(key_buf).unwrap() {
            // no match at the current key position,
            // so read the value size, and skip ahead
            // to the next key/value array
            _ = read(&fd, size_buf)?;
            sizer.clone_from_slice(size_buf);
            lseek(&fd, i64::from_ne_bytes(sizer), Whence::SeekCur)?;
            nbytes = read(&fd, key_buf)?;
        } else {
            // matched, so get the value size, to read and return the value bytes
            _ = read(&fd, size_buf)?;
            sizer.clone_from_slice(size_buf);
            let val_buf: &mut [u8] = &mut vec![0; usize::from_ne_bytes(sizer)];
            _ = read(&fd, val_buf)?;
            let mut result = Vec::new();
            result.extend_from_slice(val_buf);
            return Ok(Some(result));
        }
    }

    close(fd)?;
    Ok(None)
}

pub fn write_key_val(filepath: String, key: &str, val: &[u8]) -> Result<usize, Errno> {
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

    let nbytes = write(&fd, buffer)?;
    close(fd)?;
    Ok(nbytes)
}
