use coat_check::file_syscalls::{compact, delete_key, read_key, write_key_val};
use coat_check::fork_syscalls::size;
use coat_check::server::Server;
use coat_check::signal_syscalls::register_compaction_sig_handler;
use log::{error, info};
use nix::errno::Errno;
use std::env;

fn main() {
    env_logger::init();
    let file_folder =
        std::env::var("COAT_CHECK_FILE_PATH").expect("env var 'COAT_CHECK_FILE_PATH' not defined");

    // before: take the size of the date file, as a fork call to `wc`
    let f = file_folder.clone();
    size(f.clone());

    // allow compaction to be requested by a signal (SIGUSR2)
    match register_compaction_sig_handler() {
        Ok(_) => info!("signal handler registered!"),
        Err(e) => error!("signal handler register error -> {:#?}", e),
    };

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && &args[1] == "server" {
        let server = Server {
            port: 5000,
            filepath: f.clone(),
        };
        match server.start() {
            Ok(_) => {
                info!("server mode");
                std::process::exit(0)
            }
            Err(e) => {
                error!("syscall error {:#?}", e);
                std::process::exit(1);
            }
        }
    } else if args.len() == 2 && &args[1] == "compact" {
        match compact(file_folder.clone()) {
            Ok(_) => {
                info!("compact complete");
                std::process::exit(0)
            }
            Err(e) => match e {
                Errno::ENOENT => {
                    info!("nothing to compact, no data in {:#?}", file_folder);
                    std::process::exit(0)
                }
                _ => {
                    error!("compact error {:#?}", e);
                    std::process::exit(1)
                }
            },
        }
    } else if args.len() < 3 {
        let prog = &args[0];
        error!("Usage:\n\n{prog} <server> | compact | <(get|set) [key] [value (only with 'set')]>");
        std::process::exit(0);
    }

    let action = &args[1]; // "get", "set", or "del"
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
        "del" => match delete_key(file_folder.clone(), &args[2]) {
            Ok(bytes) => match bytes {
                Some(result) => info!("success: deleted value -> {:?}", String::from_utf8(result)),
                None => info!("no match found"),
            },
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
