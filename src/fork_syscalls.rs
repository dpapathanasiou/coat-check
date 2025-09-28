use nix::{
    sys::wait::{WaitStatus, waitpid},
    unistd::{ForkResult, execve, fork},
};
use std::ffi::{CStr, CString};

pub fn wc(file: String) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "fork(wc): parent pid {} -> child pid {}",
                nix::unistd::getpid(),
                child
            );
            match waitpid(child, None) {
                Ok(WaitStatus::Exited(pid, status)) => println!(
                    "fork(wc): in parent -> child pid {} exited, status = {:?}",
                    pid, status
                ),
                Ok(status) => {
                    println!("fork(wc): in parent -> child exited, status = {:?}", status)
                }
                Err(e) => eprintln!("fork(wc): in parent -> error waiting for child: {:?}", e),
            }
        }
        Ok(ForkResult::Child) => {
            eprintln!("fork(wc): in child -> pid {}", nix::unistd::getpid());

            let path = CString::new("/usr/bin/wc").unwrap();
            let arg1 = CString::new("wc").unwrap();
            let arg2 = CString::new("-c").unwrap();
            let arg3 = CString::new(file).unwrap();
            let args = &[arg1.as_c_str(), arg2.as_c_str(), arg3.as_c_str()];
            let env: &[&CStr] = &[];

            match execve(&path, args, env) {
                Ok(_) => unreachable!(), // if successful, will never return
                Err(e) => {
                    eprintln!("fork(wc): in child ->Child process: error {:?}", e);
                    std::process::exit(1);
                }
            };
        }
        Err(e) => eprintln!("fork(wc): error {:?}", e),
    }
}

pub fn size(file: String) {
    // syntactic sugar for the fork to wc
    wc(file)
}
