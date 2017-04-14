# ppbert

A simple command-line utility to pretty print structures encoded using
Erlang's [External Term Format](http://erlang.org/doc/apps/erts/erl_ext_dist.html).
The input is read from *stdin* or a file and written to *stdout*, making ppbert
a good candidate for shell pipelines.

At the moment, ppbert supports only a subset of the field types of the
External Term Format:

- integers;
- big integers;
- floats;
- atoms (latin-1 and UTF-8);
- binaries;
- tuples;
- lists.

## Usage

```
$ ppbert -h
ppbert 0.1.3
Vincent Foley
Pretty print structure encoded in Erlang's External Term Format

USAGE:
    ppbert [<BERT FILE>]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <BERT FILE>...

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

- Add flags to control the pretty printing (e.g., indentation width,
  number of basic values on a single line, etc.).
- Add a [jq](https://stedolan.github.io/jq/)-like query language.
- Write a man page.
