use crate::hasher;
use nix::errno::Errno;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};
use std::os::unix::io::OwnedFd;

fn derive_filepath(folder: String, name: &str) -> String {
    String::from(format!("/{}/{}.coat-check", folder, name))
}

pub fn read_key(folder: String, key: &str, buffer: &mut [u8]) -> Result<usize, Errno> {
    let filepath = derive_filepath(folder, key);
    let fd: OwnedFd = open(filepath.as_str(), OFlag::O_RDONLY, Mode::empty())?;
    let nbytes = read(&fd, buffer)?;
    close(fd)?;
    Ok(nbytes)
}

pub fn write_key_val(folder: String, key: &str, val: &[u8]) -> Result<usize, Errno> {
    let filepath = derive_filepath(folder, key);
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
