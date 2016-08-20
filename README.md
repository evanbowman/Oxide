# Oxide

## Introduction
Oxide (ox) is a simple command line pattern matching tool written in Rust. Like Ack, it processes files faster than grep by mmap'ing them, searches recursively by default, and features syntax highlighting. It forgoes Ack's whitelist design, instead searching all non-hidden files, making it more useful as a general purpose tool. ox also searches files in parallel by default, making it usually several times faster than Ack.

## Usage
Using ox is very straightforward. To do a recursive multithreaded search on an entire file tree:
```
ox <pattern>
```

In the future, the tool will eventually support command line options parsing and much of grep and ack's functionality.
