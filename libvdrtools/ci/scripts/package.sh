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
pushd libvdrtools
cp Cargo.toml Cargo.toml.backup
sed -i -E -e "s/depends = \"libvdrtools \(= [(,),0-9,.]+\),/depends = \"libvdrtools \(= ${VERSION}\),/g" Cargo.toml
sed -i -E -e "s/provides = \"libvdrtools \(= [(,),0-9,.]+\)\"/provides = \"libvdrtools \(= ${VERSION}\)\"/g" Cargo.toml
sed -i -E -e "s/provides = \"libvdrtools-dev \(= [(,),0-9,.]+\)\"/provides = \"libvdrtools-dev \(= ${VERSION}\)\"/g" Cargo.toml
cargo deb --no-build --deb-version ${VERSION} --variant libvdrtools-${PACKAGE_TYPE}
mv -f Cargo.toml.backup Cargo.toml
popd