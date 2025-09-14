use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        let prog = &args[0];
        println!("Usage:\n\n{prog} (get|set) [key] [value (only with 'set')]");
        std::process::exit(0);
    }

    let action = &args[1]; // "get" or "set"
    match action.as_str() {
        "get" => println!("get!"),
        "set" => println!("set!"),
        _ => {
            println!("error: invalid operation!");
            std::process::exit(-1);
        }
    }
    //let key_string = &args[2];
    //let val_string = &args[3]; // only if action is "set"

    // TODO: apply file syscall read/write example from https://howtorust.com/mastering-unix-system-calls-with-rusts-nix-crate/
}
