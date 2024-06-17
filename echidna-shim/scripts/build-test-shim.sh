#! /usr/bin/env bash

# The is just a temporary setup for testing. Eventually, this crate will only
# build the regular binary target and the main opener-generator app will create
# the app bundle at runtime.

set -eu -o pipefail

USAGE="build-test-shim.sh [--debug | --release]"
if [[ $# -gt 1 ]]; then
    echo "Too many arguments" >&2
    echo "${USAGE}"
    exit 1
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

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
SHIM_DIR="${SCRIPT_DIR}/.."

if [[ $CARGO_BUILD_FLAG = "" ]]; then
    # Cargo doesn't like empty arguments
    cargo build
else
    cargo build "${CARGO_BUILD_FLAG}"
fi


TARGET_DIR="${SHIM_DIR}/../target"
APP_DIR="${TARGET_DIR}/${BUILD_KIND}/TestEchidnaShim.app"

rm -r "${APP_DIR}" &>/dev/null || true # Neccesary for MacOS to pick up changes to Info.plist
mkdir "${APP_DIR}"
mkdir "${APP_DIR}/Contents"
mkdir "${APP_DIR}/Contents/MacOS"
mkdir "${APP_DIR}/Contents/Resources"

cp "${TARGET_DIR}/${BUILD_KIND}/echidna-shim" "${APP_DIR}/Contents/MacOS/TestEchidnaShim"
cp "${SHIM_DIR}/app_files/test_Info.plist" "${APP_DIR}/Contents/Info.plist"
cp "${SHIM_DIR}/app_files/test_config.json5" "${APP_DIR}/Contents/Resources/config.json5"

