#! /usr/bin/env bash

set -eu -o pipefail

USAGE="build-app.sh [--debug | --release]"

if [[ $# -gt 1 ]]; then
    echo "Too many arguments" >&2
    echo "${USAGE}"
    exit 1
fi

RELEASE=
BUILD_FLAG=debug
if [[ $# -lt 1 ]]; then
    RELEASE=
    BUILD_FLAG="--debug"

elif [[ $1 = "--debug" ]]; then
    RELEASE=
    BUILD_FLAG="--debug"

elif [[ $1 = "--release" ]]; then
    RELEASE=true
    BUILD_FLAG="--release"

else
    echo "Invalid first argument \"${1}\"" >&2
    echo "${USAGE}"
    exit 1
fi

cargo build ${RELEASE:+"--release"}
./echidna/scripts/make-app.sh "$BUILD_FLAG"

