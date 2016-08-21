use std::env;

mod dir;
mod search;

fn main() {
    let args: Vec<String> = env::args().collect();
    let root = String::from(".");
    if args.len() == 1 {
        println!("Usage: ox <pattern> <flags...>");
    } else if args.len() == 2 {
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

    }
}
