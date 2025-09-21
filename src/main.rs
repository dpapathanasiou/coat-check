use log::{error, info};
use nix::errno::Errno;
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};
use std::env;
use std::os::unix::io::OwnedFd;
use std::str::from_utf8;

fn derive_filepath(folder: String, name: &str) -> String {
    String::from(format!("/{}/{}.coat-check", folder, name))
}

fn read_key(folder: String, key: &str, buffer: &mut [u8]) -> Result<usize, Errno> {
    let filepath = derive_filepath(folder, key);
    let fd: OwnedFd = open(filepath.as_str(), OFlag::O_RDONLY, Mode::empty())?;
    let nbytes = read(&fd, buffer)?;
    close(fd)?;
    Ok(nbytes)
}

fn write_key_val(folder: String, key: &str, val: &str) -> Result<usize, Errno> {
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

fn main() {
    env_logger::init();
    let file_folder =
        std::env::var("COAT_CHECK_FILE_PATH").expect("env var 'COAT_CHECK_FILE_PATH' not defined");

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        let prog = &args[0];
        error!("Usage:\n\n{prog} (get|set) [key] [value (only with 'set')]");
        std::process::exit(0);
    }

    let action = &args[1]; // "get" or "set"
    match action.as_str() {
        "get" => {
            // TODO: size the read buffer according to the value size like this, once can get it from the serialize struct (upcoming logic change)
            //let buf_size = &args[2].chars().count();
            //let buffer: &mut [u8] = &mut vec![0; *buf_size];
            // in the meantime, an arbitrary 256 size is ok
            let buffer: &mut [u8] = &mut [0; 256];
            match read_key(file_folder, &args[2], buffer) {
                Ok(bytes) => info!(
                    "success: wrote {bytes} bytes: {:?}",
                    from_utf8(buffer).unwrap()
                ),
                Err(e) => {
                    error!("syscall error {e}");
                    std::process::exit(-1);
                }
            }
        }
        "set" => match write_key_val(file_folder, &args[2], &args[3]) {
            Ok(bytes) => info!("success: wrote {bytes} bytes"),
            Err(e) => {
                error!("syscall error {e}");
                std::process::exit(-1);
            }
        },
        _ => {
            error!("error: invalid operation!");
            std::process::exit(-1);
        }
    }
}
