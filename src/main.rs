use coat_check::file_syscalls::{read_key, write_key_val};
use log::{error, info};
use std::env;
use std::str::from_utf8;

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
        "set" => match write_key_val(file_folder, &args[2], &args[3].as_bytes()) {
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
