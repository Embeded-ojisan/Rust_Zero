mod engine;
mod helper;

use helper::DynError;
use std::{
    env,
    fs::File,
    io{BufRead, BufReader},
};

fn main() -> Result<(), DynError> {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 2 {
        eprintln!("usage: {} regex file", args[0]);
    }
}
