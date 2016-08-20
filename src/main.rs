extern crate getopts;

use std::env;
use getopts::Options;

mod dir;
mod search;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Usage: ox <pattern> <flags...>");
    } else if args.len() == 2 {
        let root = String::from(".");
        let res = dir::get_entries(&root, true);
        match res {
            Ok(entries) => {
                let pattern = args[1].clone();
                search::run_search(pattern, entries);
            }
            _ => {
                println!("Error: Could not get directory listing.");
                return;
            }
        }
    } else {
        let mut opts = Options::new();
    }
}
