#!/usr/bin/env bash

set -e

[[ $PWD = */gupax ]]

RUSTFLAGS="-C target-cpu=native" cargo build --profile optimized && du -hs target/optimized/gupax
