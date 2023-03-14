#!/usr/bin/env bash
mkdir -p artifacts
name=$(echo "$1" | cut -f 1 -d '.')
tools/ebpf-tools -a -o artifacts/$name.bin $1
tools/verifier artifacts/$name.bin $2
