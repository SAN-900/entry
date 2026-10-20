#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
compression_manifest="$(find "$cargo_home/registry/src" -path '*/spl-account-compression-1.0.0/Cargo.toml' -print -quit)"
if [[ -z "$compression_manifest" ]]; then
  echo "spl-account-compression 1.0.0 source is not in the Cargo cache" >&2
  exit 1
fi

build_dir="$(mktemp -d /tmp/entry-registry-cpi.XXXXXX)"
trap 'rm -rf "$build_dir"' EXIT

cp -a "$(dirname "$compression_manifest")" "$build_dir/spl-account-compression"
cargo-build-sbf \
  --manifest-path "$build_dir/spl-account-compression/Cargo.toml" \
  --sbf-out-dir "$build_dir" \
  --skip-tools-install \
  --offline
cargo-build-sbf \
  --manifest-path "$repo_root/programs/entry/Cargo.toml" \
  --sbf-out-dir "$build_dir" \
  --skip-tools-install \
  --offline

cd "$repo_root"
BPF_OUT_DIR="$build_dir" cargo test -p entry --test registry_cpi --offline -- --ignored
