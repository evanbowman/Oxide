extern crate regex;
extern crate num_cpus;
extern crate memmap;
extern crate time;
extern crate ansi_term;

use std::fs::DirEntry;
use std::sync::{Arc, Mutex};
use std::thread;
use self::memmap::{Mmap, Protection};
use self::ansi_term::{Style, Colour};
use self::regex::bytes::Regex;
use std::path::PathBuf;
use std::str;

pub fn run_search(pattern: String, entries: Vec<DirEntry>) {
    let num_entries = entries.len();
    let num_cores = num_cpus::get();
    let shared_entries = Arc::new(entries);
    let shared_pattern = Arc::new(pattern);
    let mut threads = Vec::new();
    let start_time = time::precise_time_ns();
    let mutex = Arc::new(Mutex::new(false));
    for idx in 0..num_cores {
        let child_entries = shared_entries.clone();
        let child_pattern = shared_pattern.clone();
        let child_mutex = mutex.clone();
        let block_size = num_entries / num_cores;
        let mut range = (idx * block_size, (idx + 1) * block_size - 1);
        if idx == num_cores {
            // This corrects for round-off error in integer division
            range.1 = num_entries - 1;
        }
        threads.push(thread::spawn(move || {
            run_ranged_search(&child_pattern, &child_entries, range, &child_mutex);
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
    println!("[Completed search in {0} ns using {1} threads]", elapsed, num_cores);
}

fn run_ranged_search(pattern: &String, entries: &Vec<DirEntry>, range: (usize, usize), mutex: &Mutex<bool>) {
    let ret = Regex::new(pattern);
    match ret {
        Err(_) => println!("Failed to construct regular expression."),
        Ok(regex) => {
            for idx in range.0..range.1 {
                let entry = &entries[idx];
                let res = Mmap::open_path(entry.path(), Protection::Read);
                match res {
                    Ok(file_mmap) => {
                        let bytes: &[u8] = unsafe { file_mmap.as_slice() };
                        let matches = search_mmap(bytes, &regex);
                        print_results(bytes, matches, entry.path(), mutex);
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

fn search_mmap(bytes: &[u8], regex: &Regex) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    for pos in regex.find_iter(bytes) {
        matches.push(pos);
    }
    return matches;
}

fn print_results(bytes: &[u8], matches: Vec<(usize, usize)>, path: PathBuf, mutex: &Mutex<bool>) {
    if matches.len() == 0 {
        return;
    }
    let lock_grd = mutex.lock().unwrap();
    let fname = path.file_name().unwrap().to_str().unwrap();
    println!("{}", Style::new().bold().paint(fname));
    for matched_pattern in matches {
        // TODO: Slice from beginning index to the previous newline, or idx 0, make str
        let matched_pattern_slice = &bytes[matched_pattern.0..matched_pattern.1];
        let matched_string = str::from_utf8(&matched_pattern_slice).unwrap();
        // TODO: Slice from ending index to the next newline or EOF
        println!("\t{}", Style::new().on(Colour::Green).fg(Colour::Black).paint(matched_string));
    }
    print!("\n");
}
