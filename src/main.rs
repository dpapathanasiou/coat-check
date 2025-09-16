use nix::errno::Errno;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};
use std::env;
use std::os::unix::io::OwnedFd;
use std::str::from_utf8;

fn derive_filepath(name: &str) -> String {
    String::from(format!("/tmp/{}.coat-check", name))
}

fn read_key(key: &str) -> Result<String, Errno> {
    let filepath = derive_filepath(key);
    let buffer: &mut [u8] = &mut [0; 256];
    let fd: OwnedFd = open(filepath.as_str(), OFlag::O_RDONLY, Mode::empty())?;
    let _ = read(&fd, buffer)?;
    let data = String::from(from_utf8(buffer).unwrap());
    close(fd)?;
    Ok(data)
}

fn write_key_val(key: &str, val: &str) -> Result<usize, Errno> {
    let filepath = derive_filepath(key);
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        let prog = &args[0];
        println!("Usage:\n\n{prog} (get|set) [key] [value (only with 'set')]");
        std::process::exit(0);
    }

    let action = &args[1]; // "get" or "set"
    match action.as_str() {
        "get" => match read_key(&args[2]) {
            Ok(data) => println!("success: {data}"),
            Err(e) => {
                println!("syscall error {e}");
                std::process::exit(-1);
            }
        },
        "set" => match write_key_val(&args[2], &args[3]) {
            Ok(bytes) => println!("success: wrote {bytes} bytes"),
            Err(e) => {
                println!("syscall error {e}");
                std::process::exit(-1);
            }
        },
        _ => {
            println!("error: invalid operation!");
            std::process::exit(-1);
        }
    }
}
