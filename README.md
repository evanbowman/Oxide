# Oxide

## Introduction
Oxide (ox) is a simple pattern matching tool inspired by Ack and Grep. It achieves even faster searches by leveraging hardware concurrency, mmap'ing files rather than buffering them, and ignoring hidden files (like git repositories). I've found that my implemenatation is about three times faster than Ack and 90 times faster than grep for searching code bases. It's name is also 33% shorter than Ack, and 50% shorter than grep, and is therefore faster to type.

## Usage
Using ox is very straightforward. To do a multithreaded recursive search of a file tree:
```
ox <pattern>
```

In the future, the tool will eventually support command line options parsing and much of grep and ack's functionality.
