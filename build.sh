#!/usr/bin/env bash
set -euo pipefail

cargo build --release --locked
case "$(uname -s)" in
  Darwin) library=libcalcit_fswatch.dylib ;;
  Linux) library=libcalcit_fswatch.so ;;
  *) echo "unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac
mkdir -p dylibs
cp "target/release/$library" dylibs/
