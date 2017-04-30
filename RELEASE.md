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
