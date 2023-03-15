#!/usr/bin/env bash
mkdir -p artifacts
name=$(echo $1 | cut -f 1 -d '.')
./ebpf-vc $1 > artifacts/$name.mlw
why3 --debug=ignore_unused_vars prove -P CVC4,1.8 artifacts/$name.mlw
