#!/bin/zsh
/usr/local/bin/qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-storm_g4.bin -smp 4