extern crate regex;
extern crate num_cpus;
extern crate memmap;

use std::fs::DirEntry;
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use self::memmap::Mmap;

struct Range {
    begin: usize,
    end: usize,
}

pub fn find_occurences(pattern: String, files: Vec<DirEntry>) {
    let num_files = files.len();
    let num_cores = num_cpus::get();
    let shared_fnames = Arc::new(files);
    let shared_pattern = Arc::new(pattern);
    let mut threads = Vec::new();
    for idx in 0..num_cores {
        let child_fnames = shared_fnames.clone();
        let child_pattern = shared_pattern.clone();
        let block_size = num_files / num_cores;
        let range = Range {
            begin: idx * block_size,
            end: (idx + 1) * block_size - 1
        }; // TODO: Urgent: fix ranges
        threads.push(thread::spawn(move || {
            run_search(&child_pattern, &child_fnames, range);
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
}

fn run_search(pattern: & String, files: & Vec<DirEntry>, range: Range) {
    println!("My range is {0} {1}", range.begin, range.end);
    // 1. Map view of each file
    // 2. Search files with regex
    // 3. Print results while searching
}
