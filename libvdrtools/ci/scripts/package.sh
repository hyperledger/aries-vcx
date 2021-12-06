#!/bin/bash

if [ $# -ne 1 ]
  then
    echo "ERROR: Incorrect number of arguments"
    echo "Usage:"
    echo "$0 <version>"
    exit 1
fi

BASE_VERSION=$1

set -eux

PACKAGE_TYPE=$(lsb_release -cs)
# REVISION=$(git rev-parse HEAD | cut -c 1-7)
VERSION=${BASE_VERSION}-${PACKAGE_TYPE}  # TODO: Autodetect main part
pushd libindy
cargo deb --no-build --deb-version ${VERSION} --variant libvdrtools-${PACKAGE_TYPE}
popd