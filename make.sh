#!/bin/bash
set -eo pipefail



case "$1" in
	all | '' )
		./make.sh darwin
		./make.sh linux
		;;
	darwin )
		SDKROOT=$(realpath ../zig-build-macos-sdk) \
		MACOSX_DEPLOYMENT_TARGET=14.0 \
		cargo zigbuild \
			--target aarch64-apple-darwin \
			--release
		;;
	linux )
		targets=(
			x86_64-unknown-linux-musl
		)
			# x86_64-pc-windows-gnu
		for t in "${targets[@]}"; do
		echo "Building $t"
		cargo build --release --target "$t"
		done
		;;

	release | release/ )
		cp target/x86_64-unknown-linux-musl/release/imapdummyrs release/imapdummyrs-linux-musl-amd64
		cp target/aarch64-apple-darwin/release/imapdummyrs release/imapdummyrs-darwin-arm64
		realpath release
		;;
esac

