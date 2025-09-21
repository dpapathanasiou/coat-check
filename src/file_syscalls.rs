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

pub fn write_key_val(folder: String, key: &str, val: &str) -> Result<usize, Errno> {
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
    let nbytes = write(&fd, val.as_bytes())?;
    close(fd)?;
    Ok(nbytes)
}
