#!/bin/sh

cd $(dirname $0)

clang -g hello.c -o data/hello
clang -g segv.c -o data/segv
