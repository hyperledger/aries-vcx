#!/bin/bash

if [ $# -ne 2 ]
  then
    echo "ERROR: Incorrect number of arguments"
    echo "Usage:"
    echo "$0 <version> <rpm build num>"
    exit 1
fi

pushd libindy

base_version=$1
number=$2
dir=$(pwd)
result_dir=$(pwd)/rpms
set -eux

version=${base_version}

sed \
	-e "s|@version@|$version|g" \
	-e "s|@dir@|$dir|g" \
	-e "s|@release@|$number|g" \
	-e "s|@result_dir@|$result_dir|g" \
    rpm/libvdrtools.spec.in > libvdrtools.spec

mkdir ${result_dir}

fakeroot rpmbuild -ba libvdrtools.spec --nodeps || exit 7
popd