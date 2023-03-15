# `ebpf-vc` - Verification condition generation for eBPF

Requirements:

- Rust version 1.68
- Why3 version 1.5.0 (for passing the verification condition output to)


Build:

    cargo build --release

Run:

    cargo run --release [file]

See the shell scripts in `util` for usage examples.
