#!/bin/bash

executable="./target/debug/simulador_de_fluxo"

cargo b

perf record --call-graph dwarf "$executable"
perf script | inferno-collapse-perf 1> stacks.folded 2>/dev/null

bat stacks.folded | inferno-flamegraph > profile.svg
