#! /usr/bin/env bash

# The is just a temporary setup for testing. Eventually, this crate will only
# build the regular binary target and the main opener-generator app will create
# the app bundle at runtime.

set -eu -o pipefail

USAGE="build.sh [--debug | --release]"
if [[ $# -gt 1 ]]; then
    echo "Too many arguments" >&2
    echo "${USAGE}"
fi

BUILD_KIND=""
CARGO_BUILD_FLAG=""
if [[ $# -lt 1 ]]; then
    BUILD_KIND="debug"
elif [[ $1 = "--debug" ]]; then
    BUILD_KIND="debug"
elif [[ $1 = "--release" ]]; then
    BUILD_KIND="release"
    CARGO_BUILD_FLAG="--release"
else
    echo "Invalid first argument \"${1}\"" >&2
    echo "${USAGE}"
    exit 1
fi

if [[ $CARGO_BUILD_FLAG = "" ]]; then
    # Cargo doesn't like empty arguments
    cargo build
else
    cargo build "${CARGO_BUILD_FLAG}"
fi

./make-app.sh "--${BUILD_KIND}"

