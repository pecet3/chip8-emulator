use std::{env, process};

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("ERROR! not enough args");
        process::exit(1);
    }
}
