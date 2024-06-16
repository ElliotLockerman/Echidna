#! /usr/bin/env bash
# The is just a temporary setup for testing. Eventually, this crate will only
# build the regular binary target and the main opener-generator app will create
# the app bundle at runtime.

set -eu -o pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
cd "${SCRIPT_DIR}/.."

cargo clean
rm -r "TestEchidnaShim.app" || true

