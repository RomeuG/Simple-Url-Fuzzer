simple-url-fuzzer
=================

This is a simple URL Fuzzer made in C++.

Features
========

- Multithreading
- Use `@@` to identify which part of the URL will be fuzzed
- Custom extension A.K.A. a string that can be concatenated to the end of each word

Usage
=====

```
Usage: suf [OPTION...] [URL]...
Software to do some URL Fuzzing.

  -e, --extension=value      Add value at the end of each word
  -m, --timeout=value        Timeout value
  -t, --threads=value        Number of threads
  -u, --url=value            Url to fuzz
  -v, --verbose              Verbose
  -w, --wordlist=value       Wordlist with 1 string per line
  -?, --help                 Give this help list
      --usage                Give a short usage message
  -V, --version              Print program version
```

Example:

`$ ./suf -u "https://www.example.com/@@" -w wordlists/wordlist.txt -t 64 -e ".php"`
