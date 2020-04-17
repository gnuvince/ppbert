# 0.9.3
- BUGFIX: Properly escape double quotes and backslashes in binary and
  string literals
- FEATURE: Display the atoms true and false as the JSON booleans
  rather than the strings "true" and "false

# 0.9.2
- Add a GitHub Actions pipeline to create Linux, macOS and Windows binaries

# 0.9.0
- `bert-convert` has been removed; I never used it, and it wasn't getting the love it deserved.
- `ppbert` now uses `getopts` for command-line parsing.
- Removing `rayon` and `clap` makes builds much faster, a nearly 3x speed-up on my machine.


# 0.8.5
## Bug fixes
- Call `Write::flush()` to ensure that IO errors are propagated; necessary to exit early when a pipe is closed.  This causes a performance regression from 0.8.4.
## Performance improvements
- Use `ryu` to print floating point numbers.

# 0.8.4
## Bug fixes
  - Ensure that `ppbert` exits when a BrokenPipe error is encountered.
## Performance improvements
  - Wrap `stdout` in a `BufWriter`; this improves the performance of outputting as Erlang by 2x and JSON by 1.15x.


# 0.8.3
## New features
  - The parsing and the prettying printing for .bert2 and disk_log files are interleaved
  - The filename is included in verbose messages (e.g., how long the parsing took)


# 0.8.2
## New features
  - Add `-t` as a short-hand for `--transform-proplists`


# 0.8.0
## API change
  - Replace `BertTerm::dump_term` with `BertTerm::write_as_bert`.
## Other changes
  - Add API documentation in bertterm


# 0.7.0
## Performance improvement
  - Use `itoa` crate to write integers a bit faster.
  - Use `fs::read` to read input files almost two times faster.
  - Remove some heap allocations.
## API change
  - Simplify pretty printing API: `BertTerm` has two new functions, `write_as_erlang` and `write_as_json`, they accept an io::Write object.
  - The structs `PrettyPrinter` and `JsonPrettyPrinter` are no longer public.


# 0.6.1
## Performance improvement
  - A number of unsafe function usages (e.g., `Vec::set_len()`) have been removed and replaced with safer alternatives.  It turns out that it gives a slight increase in the performance of parsing. Win-win!


# 0.6.0
## New features
  - Support for `disk_log` files.  The Erlang module, `disk_log`, writes BERT-encoded terms to files on disk; the new `--disk-log` (`-d`) flag allows ppbert to read this file format.


# 0.5.2
## Performance improvement
  - Nearly halved the time it takes to pretty print my benchmark .bert files by manually buffering the output of strings and binaries.


# 0.5.1
## Tooling changes
  - Added a small shell script, build_musl.sh, to create a musl binary.


# 0.5.0
## Interface changes
  - clap's extra features (e.g., flag name suggestions) have been removed.
  - A warning's format was changed to conform to usual Unix style.
  - The 's' (for 'seconds') after times in verbose mode has been removed.
  - The time required to read a file has been added to ppbert's verbose mode.


# 0.4.2
## Bug fixes
  - A parse error in bert-convert will show an error message rather than panic.
  - Piping ppbert into another command will not cause a broken pipe error if the stream is not consumed entirely.


# 0.4.1
## New features
  - A new binary, bert-convert, was added to convert bertconf's .bert files to rig's .bert2 files.


# 0.4.0
## Performance improvements
  - A number of micro-optimizations have been applied to the parser; in our tests, parsing is now nearly twice as fast as before.
## Interface changes
  - BertError::EOF has been removed and replaced with BertError::NotEnoughData; this new error contains more information: the number of bytes that needed to be read, and the number of bytes that were available.


# 0.3.1
## Bug fixes
  - Ensure that only strings, binaries, and atoms are used as keys in a proplist.


# 0.3.0
## New features
  - The `-j/--json` flag can be used to output JSON rather than Erlang terms.
  - The `--transform-proplists` flag can be used to output Erlang proplists as JSON objects.
## Interface changes
  - The `-s/--skip-pretty-print` has been renamed to `-p/--parse`.


# 0.2.6
## Bug fixes
  - fix bug where `--verbose` would not print the parse time if `--skip-pretty-print` was set.


# 0.2.5
## New features
  - add support for .bert2 files.
## Bug fixes
  - fix the times reported by the `--verbose` flags were wrong; the leading zeros were missing.


# 0.2.4
## New features
  - add command-line flag (-v) to show how long each phase (parsing and pretty printing) takes.
  - add command-line flag (-s) to stop after parsing.
## Improvements
  - improve performance of the parsing methods for atoms,
      strings, and binaries by avoiding bounds checking.


# 0.2.3
  - Improve performance of pretty printer


# 0.2.2
## New features
  - add command-line option (-i) to control the indentation width
  - add command-line option (-m) to control the maximum number of basic terms on a single line
  - update manpage to reflect new command line options
  - update manpage to describe the supported term types


# 0.2.1
## Bug fixes
  - ppbert 0.2.0 showed its version number as "0.1.3"; now it prints the version defined in Cargo.toml


# 0.2.0
## New features
  - maps are now supported
  - nil (empty list) now has its own BertTerm item
  - ppbert returns 0 when all files have successfully parsed, 1 if at least one parsed incorrectly
  - ppbert now has a man page
  - ppbert now has release notes


# 0.1.3
## New features
  - ppbert now accepts filenames on the command-line, following the convention of most Unix tools
## Bug fixes
  - escaped characters are now correctly printed in hexadecimal notation
