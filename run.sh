#!/usr/bin/env bash

rm -f test.psm
LD_LIBRARY_PATH=./psmalloc/build cargo run --example $@
