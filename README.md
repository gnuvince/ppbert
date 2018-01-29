# Table of contents

- [ppbert](#ppbert)
- [bert-convert](#bert-convert)

# ppbert

A command-line utility to pretty print structures encoded using
Erlang's [External Term Format](http://erlang.org/doc/apps/erts/erl_ext_dist.html).
The input is read from *stdin* or a file and written to *stdout*,
making ppbert a good candidate for shell pipelines.

At the moment, ppbert supports the following subset of the External
Term Format:

- Small integers (tag: 97);
- Integers (tag: 98);
- Floating-point numbers (tags: 70, 99);
- Big integers (tags: 110, 111);
- Latin-1 atoms (tags: 100, 115);
- UTF-8 atoms (tags: 118, 119);
- Strings (tag: 107);
- Binaries (tag: 109);
- Tuples (tags: 104, 105);
- Lists (tags: 106, 108);
- Maps (tag: 116).


## Usage

```
$ ppbert -h
ppbert 0.4.0
Vincent Foley
Pretty print structure encoded in Erlang's External Term Format

USAGE:
    ppbert [FLAGS] [OPTIONS] [FILES]...

FLAGS:
    -2, --bert2                  Parse .bert2 files
    -h, --help                   Prints help information
    -j, --json                   Output in JSON
    -p, --parse                  Parse the input, don't pretty print it
        --transform-proplists    Transform proplists into JSON objects (only valid with --json)
    -V, --version                Prints version information
    -v, --verbose                Enable verbose mode

OPTIONS:
    -i, --indent-width <num>          Number of spaces to indent nested terms
    -m, --max-terms-per-line <num>    Number of spaces to indent nested terms

ARGS:
    <FILES>...


$ ppbert mini_dict.bert
[
  {host, "localhost"},
  {port, 80},
  {
    headers,
    [
      {
        <<"X-Real-Ip">>,
        {127, 0, 0, 1}
      },
      {<<"Keep-alive">>, true}
    ]
  }
]

$ printf "\x83\x77\x04atom" | ppbert
atom
```

## Performance

Ppbert is written in Rust and offers an appreciable performance gain
over using Erlang's `erlang:binary_to_term/1` and `io:format/2`.

```sh
$ cat erl_ppbert
#!/usr/bin/env escript

main(Args) ->
    lists:foreach(fun (Filename) ->
        {ok, Binary} = file:read_file(Filename),
        io:format("~p~n", [binary_to_term(Binary)])
    end, Args).

$ du large.bert
96M     large.bert

$ time ./erl_ppbert large.bert >/dev/null

real    1m10.968s
user    0m49.644s
sys     0m13.628s

$ time ppbert large.bert >/dev/null

real    0m6.902s
user    0m6.116s
sys     0m0.452s
```

## Future work

- ~~Add flags to control the pretty printing (e.g., indentation width, number of basic values on a single line, etc.).~~
- Add a [jq](https://stedolan.github.io/jq/)-like query language.
- ~~Write a man page.~~


# bert-convert

A command-line utility to convert a bert file encoded in
[bertconf's](https://github.com/ferd/bertconf) DB format to
[rig's](https://github.com/lpgauth/rig) format.

## Usage

```
$ bert-convert -h
bert-convert 0.4.0
Vincent Foley
Convert bertconf .bert files to rig .bert2 files

USAGE:
    bert-convert [FLAGS] [OPTIONS] [FILES]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Enables verbose mode

OPTIONS:
    -d, --output-dir <DIR>    Selects the output directory [default: .]

ARGS:
    <FILES>...

$ bert-convert -v -d /tmp/bert2 large.bert
bert-convert: large.bert: Parse time: 0.556925379s; Dump time: 0.580329024s

$ ls /tmp/bert2
large.bert2
```
