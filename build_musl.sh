#!/bin/bash

docker container run     							\
       --rm              							\
       -v "$PWD":/volume 							\
       -e CARGO_TARGET_DIR="/volume/musl-target"    \
       clux/muslrust     							\
       cargo build --release
