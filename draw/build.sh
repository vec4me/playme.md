#!/usr/bin/sh

set -e
cd $(dirname $0)

clang *.c -Weverything -std=c99 -lm -o render