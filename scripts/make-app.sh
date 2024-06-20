#! /usr/bin/env bash

set -eu -o pipefail

USAGE="make-app.sh [--debug | --release]"
if [[ $# -gt 1 ]]; then
    echo "Too many arguments" >&2
    echo "${USAGE}"
    exit 1
fi

BUILD_MODE=""
if [[ $# -lt 1 ]]; then
    BUILD_MODE="debug"

elif [[ $1 = "--debug" ]]; then
    BUILD_MODE="debug"

elif [[ $1 = "--release" ]]; then
    BUILD_MODE="release"

else
    echo "Invalid first argument \"${1}\"" >&2
    echo "${USAGE}"
    exit 1
fi

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
REPO_DIR="${SCRIPT_DIR}/.."

TARGET_DIR="${REPO_DIR}/target/${BUILD_MODE}"
APP_DIR="${TARGET_DIR}/Echidna.app"

rm -r "${APP_DIR}" &>/dev/null || true # Neccesary for MacOS to pick up changes to Info.plist
mkdir "${APP_DIR}"
mkdir "${APP_DIR}/Contents"
mkdir "${APP_DIR}/Contents/MacOS"
mkdir "${APP_DIR}/Contents/Resources"

cp "${TARGET_DIR}/echidna" "${APP_DIR}/Contents/MacOS/Echidna"
cp "${REPO_DIR}/app_files/Info.plist" "${APP_DIR}/Contents/Info.plist"
cp "${REPO_DIR}/app_files/AppIcon.icns" "${APP_DIR}/Contents/Resources/AppIcon.icns"
cp "${TARGET_DIR}/echidna-shim" "${APP_DIR}/Contents/Resources"

