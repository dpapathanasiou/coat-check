use libc::c_int;
use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};

pub static COMPACT_SIGNALED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_compaction_signal(sig: c_int) {
    println!("sig :: Received compact signal {:#?}!", sig);
    if sig == Signal::SIGUSR2 as c_int {
        COMPACT_SIGNALED.store(true, Ordering::Relaxed);
        println!(
            "sig :: COMPACT_SIGNALED contains {:#?}",
            COMPACT_SIGNALED.load(Ordering::Relaxed)
        );
    }
}

pub fn register_compaction_sig_handler() -> Result<(), Box<dyn Error>> {
    let sa = SigAction::new(
        SigHandler::Handler(handle_compaction_signal),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );

    unsafe {
        sigaction(Signal::SIGUSR2, &sa)?;
    }

    Ok(())
}
