# Table of contents

- [ppbert](#ppbert)

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
$ ppbert --help
ppbert 0.9.0

Options:
    -V, --version       display version
    -h, --help          display this help
    -i, --indent NUM    indent with NUM spaces
    -m, --per-line NUM  print at most NUM basic terms per line
    -p, --parse         parse only, not pretty print
    -2, --bert2         parse .bert2 files
    -d, --disk-log      parse disk_log files
    -v, --verbose       show diagnostics on stderr
    -j, --json          print as JSON
    -t, --transform-proplists
                        convert proplists to JSON objects


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

real	0m43.017s
user	0m38.846s
sys 	0m4.345s

$ time ppbert large.bert >/dev/null

real	0m1.802s
user	0m1.251s
sys     0m0.549s
```
