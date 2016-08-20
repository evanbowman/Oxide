extern crate regex;
extern crate num_cpus;
extern crate memmap;
extern crate time;

use std::fs::DirEntry;
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use self::memmap::{Mmap, Protection};
use self::regex::bytes::Regex;
use std::path::PathBuf;

struct Range {
    begin: usize,
    end: usize,
}

pub fn run_search(pattern: String, entries: Vec<DirEntry>) {
    let num_entries = entries.len();
    let num_cores = num_cpus::get();
    let shared_entries = Arc::new(entries);
    let shared_pattern = Arc::new(pattern);
    let mut threads = Vec::new();
    let start_time = time::precise_time_ns();
    for idx in 0..num_cores {
        let child_entries = shared_entries.clone();
        let child_pattern = shared_pattern.clone();
        let block_size = num_entries / num_cores;
        let range = Range {
            begin: idx * block_size,
            end: (idx + 1) * block_size - 1
        }; // TODO: Urgent: fix ranges! Corner case: less files than threads!!
        threads.push(thread::spawn(move || {
            run_ranged_search(&child_pattern, &child_entries, range);
        }));
    }
    for thrd in threads {
        let res = thrd.join();
        match res {
            Err(e) => {
                println!("Error: thread panicked with error code {:?}", e);
            },
            _ => {},
        }
    }
    let elapsed = time::precise_time_ns() - start_time;
    println!("[Completed search in {0}ns using {1} threads]", elapsed, num_cores);
}

fn run_ranged_search(pattern: &String, entries: &Vec<DirEntry>, range: Range) {
    let ret = Regex::new(pattern);
    match ret {
        Err(_) => println!("Failed to construct regular expression."),
        Ok(regex) => {
            for idx in range.begin..range.end {
                let entry = &entries[idx];
                let res = Mmap::open_path(entry.path(), Protection::Read);
                match res {
                    Ok(file_mmap) => {
                        let bytes: &[u8] = unsafe { file_mmap.as_slice() };
                        search_mmap(bytes, &regex, entry.path());
                    },
                    Err(_) => {
                        // TODO: don't print errors for mmaping empty files
                        println!("Error: Failed to mmap {:?}", entry.path());
                        continue;
                    },
                }
            }
        },
    }
}

fn search_mmap(bytes: &[u8], regex: &Regex, path: PathBuf) {
    for pos in regex.find_iter(bytes) {
        println!("{:?}", pos);
        // TODO: Instead push them all to a vector...
        // only print when finished with file...
        // lock mutex around printing.
        // Oh yeah also get the full line that match occurs on,
        // and print in color.
    }
}
