# 0.2.4

## New features

- add command-line flag (-v) to show how long each phase
  (parsing and pretty printing) takes.
- add command-line flag (-s) to stop after parsing.

## Improvements

- improve performance of the parsing methods for atoms,
  strings, and binaries by avoiding bounds checking.


# 0.2.3

- Improve performance of pretty printer


# 0.2.2

## New features

- add command-line option (-i) to control the indentation width
- add command-line option (-m) to control the maximum number of
  basic terms on a single line
- update manpage to reflect new command line options
- update manpage to describe the supported term types


# 0.2.1

## Bug fixes

- ppbert 0.2.0 showed its version number as "0.1.3"; now it prints the
  version defined in Cargo.toml


# 0.2.0

## New features

- maps are now supported
- nil (empty list) now has its own BertTerm item
- ppbert returns 0 when all files have successfully parsed, 1 if at
  least one parsed incorrectly
- ppbert now has a man page
- ppbert now has release notes


# 0.1.3

## New features

- ppbert now accepts filenames on the command-line, following the
  convention of most Unix tools

## Bug fixes

- escaped characters are now correctly printed in hexadecimal notation
