#!/usr/bin/env bash

set -e

[[ $PWD = */gupax ]]

if [[ $1 = *all* ]]; then
	echo "=== building all ==="
	echo "=== windows ==="
	cargo build --profile optimized --target x86_64-pc-windows-gnu
#	echo "=== macos ==="
#	cargo build --profile optimized --target x86_64-apple-darwin
	echo "=== linux ==="
	cargo build --profile optimized
	du -hs target/x86_64-pc-windows-gnu/optimized/gupax target/x86_64-apple-darwin/optimized/gupax target/optimized/gupax
else
	echo "=== building linux cpu optimized ==="
	RUSTFLAGS="-C target-cpu=native" cargo build --profile optimized
	du -hs target/optimized/gupax
fi
