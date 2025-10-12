use coat_check::file_syscalls::{read_key, write_key_val};
use coat_check::fork_syscalls::size;
use coat_check::server::start;
use log::{error, info};
use std::env;

fn main() {
    env_logger::init();
    let file_folder =
        std::env::var("COAT_CHECK_FILE_PATH").expect("env var 'COAT_CHECK_FILE_PATH' not defined");

    // before: take the size of the date file, as a fork call to `wc`
    let f = file_folder.clone();
    size(f.clone());

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && &args[1] == "server" {
        match start(5000) {
            Ok(_) => info!("ok"),
            Err(e) => {
                error!("syscall error {:#?}", e);
                std::process::exit(1);
            }
        }
    } else if args.len() < 3 {
        let prog = &args[0];
        error!("Usage:\n\n{prog} <server> | <(get|set) [key] [value (only with 'set')]>");
        std::process::exit(0);
    }

    let action = &args[1]; // "get" or "set"
    match action.as_str() {
        "get" => match read_key(file_folder.clone(), &args[2]) {
            Ok(bytes) => match bytes {
                Some(result) => info!("success: matched -> {:?}", String::from_utf8(result)),
                None => info!("no match found"),
            },
            Err(e) => {
                error!("syscall error {:#?}", e);
                std::process::exit(1);
            }
        },
        "set" => match write_key_val(file_folder.clone(), &args[2], &args[3].as_bytes()) {
            Ok(bytes) => info!("success: wrote {bytes} bytes"),
            Err(e) => {
                error!("syscall error {:#?}", e);
                std::process::exit(1);
            }
        },
        _ => {
            error!("error: invalid operation!");
            std::process::exit(1);
        }
    };

    // after: take the size of the date file, as a fork call to `wc`
    size(f.clone());
}
