#! /usr/bin/env bash
# The is just a temporary setup for testing. Eventually, this crate will only
# build the regular binary target and the main opener-generator app will create
# the app bundle at runtime.

set -eu -o pipefail

cargo clean
rm -r "TermOpenShim.app" || true

