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
    let start_time = time::precise_time_ns();
    let num_entries = entries.len();
    let num_cores = num_cpus::get();
    let mut num_threads = 0;
    if num_cores < num_entries {
        let mutex = Arc::new(Mutex::new(false));
        let shared_entries = Arc::new(entries);
        let shared_pattern = Arc::new(pattern);
        let mut threads = Vec::new();
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
                ranged_search(&child_pattern, &child_entries, range, &child_mutex);
            }));
            num_threads += 1;
        }
        for thrd in threads {
            let res = thrd.join();
            match res {
                Err(e) => {
                    println!("Error: thread panicked with error code {:?}", e);
                }
                _ => {}
            }
        }
    } else {
        num_threads = 1;
        let mutex = Mutex::new(false);
        // TODO: Make the mutex an optional argument
        ranged_search(&pattern, &entries, (0, entries.len()), &mutex);
    }
    let elapsed = time::precise_time_ns() - start_time;
    println!("[Completed search of {0} files in {1} ns using {2} thread(s)]",
             num_entries,
             elapsed,
             num_threads);
}

fn ranged_search(pattern: &String,
                 entries: &Vec<DirEntry>,
                 range: (usize, usize),
                 mutex: &Mutex<bool>) {
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
                    }
                    Err(_) => {
                        // TODO: don't print errors for mmaping empty files
                        println!("Error: Failed to mmap {:?}", entry.path());
                        continue;
                    }
                }
            }
        }
    }
}

fn search_mmap(bytes: &[u8], regex: &Regex) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    for pos in regex.find_iter(bytes) {
        matches.push(pos);
    }
    return matches;
}

#[allow(unused_variables)] // lock_grd needs to exist for thread synchronization
fn print_results(bytes: &[u8], matches: Vec<(usize, usize)>, path: PathBuf, mutex: &Mutex<bool>) {
    if matches.len() == 0 {
        return;
    }
    let lock_grd = mutex.lock().unwrap();
    let fname = path.file_name().unwrap().to_str().unwrap();
    println!("[{}]", Style::new().bold().paint(fname));
    for matched_pattern_idxs in matches {
        print!("\t");
        print_leading_context(bytes, matched_pattern_idxs);
        print_match(bytes, matched_pattern_idxs);
        print_trailing_context(bytes, matched_pattern_idxs);
        print!("{}", Style::default().paint("\n"));
    }
    print!("\n\n");
}

fn print_match(bytes: &[u8], matched_pattern_idxs: (usize, usize)) {
    let matched_pattern_slice = &bytes[matched_pattern_idxs.0..matched_pattern_idxs.1];
    let ret = str::from_utf8(&matched_pattern_slice);
    match ret {
        Ok(matched_string) => {
            print!("{}",
                   Style::new().on(Colour::Green).fg(Colour::Black).paint(matched_string));
        }
        _ => {
            let error_fmt = Style::new().bold().fg(Colour::Red);
            println!("{}",
                     error_fmt.paint("Found non-utf8 character, skipping..."));
        }
    }
}

fn print_leading_context(bytes: &[u8], matched_pattern_idxs: (usize, usize)) {
    let line_start_idx = seek_line_start(bytes, matched_pattern_idxs.0);
    if line_start_idx != matched_pattern_idxs.0 {
        let leading_slice = &bytes[line_start_idx..(matched_pattern_idxs.0)];
        let ret = str::from_utf8(&leading_slice);
        match ret {
            Ok(leading_string) => print!("{}", leading_string),
            _ => {
                let error_fmt = Style::new().bold().fg(Colour::Red);
                println!("{}",
                         error_fmt.paint("Found non-utf8 character, skipping..."));
            }
        }
    }
}

fn print_trailing_context(bytes: &[u8], matched_pattern_idxs: (usize, usize)) {
    let line_end_idx = seek_line_end(bytes, matched_pattern_idxs.1);
    if line_end_idx != matched_pattern_idxs.1 {
        let trailing_slice = &bytes[(matched_pattern_idxs.1)..line_end_idx];
        let ret = str::from_utf8(&trailing_slice);
        match ret {
            Ok(trailing_string) => print!("{}", trailing_string),
            _ => {
                let error_fmt = Style::new().bold().fg(Colour::Red);
                println!("{}",
                         error_fmt.paint("Found non-utf8 character, skipping..."));
            }
        }
    }
}

fn seek_line_start(bytes: &[u8], position: usize) -> usize {
    let mut idx = position;
    while idx != 0 {
        idx -= 1;
        if bytes[idx] == '\n' as u8 {
            idx += 1;
            break;
        }
    }
    return idx;
}

fn seek_line_end(bytes: &[u8], position: usize) -> usize {
    let mut idx = position;
    while idx != bytes.len() - 1 {
        idx += 1;
        if bytes[idx] == '\n' as u8 {
            break;
        }
    }
    return idx;
}
