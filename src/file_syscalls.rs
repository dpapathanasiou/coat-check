use crate::hasher;
use nix::errno::Errno;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};
use std::os::unix::io::OwnedFd;

pub fn read_key(filepath: String, key: &str, buffer: &mut [u8]) -> Result<usize, Errno> {
    let fd: OwnedFd = open(filepath.as_str(), OFlag::O_RDONLY, Mode::empty())?;
    let nbytes = read(&fd, buffer)?;
    close(fd)?;
    Ok(nbytes)
}

pub fn write_key_val(filepath: String, key: &str, val: &[u8]) -> Result<usize, Errno> {
    let fd: OwnedFd = open(
        filepath.as_str(),
        OFlag::O_WRONLY | OFlag::O_CREAT,
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
