#!/bin/sh

cd $(dirname $0)

clang -g hello.c -o data/hello
