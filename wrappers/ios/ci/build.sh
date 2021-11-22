#!/bin/sh

set -ex

export PKG_CONFIG_ALLOW_CROSS=1
export CARGO_INCREMENTAL=1
export RUST_LOG=indy=trace
export RUST_TEST_THREADS=1

INDY_VERSION="efb7215" # indy-1.16.0-post-59 - "v1.16.0" + rusql update fix + (number of other commits on master branch)
REPO_DIR=$PWD
SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"
OUTPUT_DIR=/tmp/artifacts
INDY_SDK_DIR=$OUTPUT_DIR/indy-sdk

setup() {
    echo "Setup rustup"
    rustup default 1.55.0
    rustup component add rls-preview rust-analysis rust-src

    echo "Setup rustup target platforms"
    rustup target add aarch64-apple-ios x86_64-apple-ios

    RUST_TARGETS=$(rustc --print target-list | grep -i ios)
    if [ "$RUST_TARGETS" = "" ]; then
        echo "Error: Rust targets for iOS has not been set! Try to run 'xcode-select -s /Applications/Xcode.app'"
        exit 1
    fi

    echo "Install Rust Xcode tools"
    cargo install cargo-lipo
    cargo install cargo-xcode

    echo "Check Homebrew"
    BREW_VERSION=$(brew --version)
    if ! [[ $BREW_VERSION =~ ^'Homebrew ' ]]; then
        echo "Error: Missing Homebrew, package manager for macOS to install native dependencies."
        exit 1
    fi

    echo "Install required native libraries and utilities"
    which pkg-config &>/dev/null || brew install pkg-config
    # Libsodium version<1.0.15 is required
    # brew install https://raw.githubusercontent.com/Homebrew/homebrew-core/65effd2b617bade68a8a2c5b39e1c3089cc0e945/Formula/libsodium.rb
    which automake &>/dev/null || brew install automake
    which autoconf &>/dev/null || brew install autoconf
    which cmake &>/dev/null || brew install cmake
    which wget &>/dev/null || brew install wget
    which truncate &>/dev/null || brew install truncate
    brew list openssl &>/dev/null || brew install openssl@1.1
    brew list zmq &>/dev/null || brew install zmq
    brew list libzip &>/dev/null || brew install libzip
    brew list tree &>/dev/null || brew install tree

    mkdir -p $OUTPUT_DIR

    # Figure out which OPENSSL we have available
    export OPENSSL_BASE_DIR="/usr/local/Cellar/openssl@1.1"
    for f in $(ls -t "$OPENSSL_BASE_DIR"); do
      local ABSOLUTE_FILE_PATH="${OPENSSL_BASE_DIR}/${f}"
      if [ -d "$ABSOLUTE_FILE_PATH" ] && [ -d "$ABSOLUTE_FILE_PATH/lib" ]; then
        export OPENSSL_VERSION=$f
        export OPENSSL_DIR=$ABSOLUTE_FILE_PATH # Used later by cyclone
        break
      fi
    done
    if [ -z "$OPENSSL_VERSION" ]; then
      echo >&2 "Error: Failed to find an OpenSSL installation in $OPENSSL_BASE_DIR"
      exit 1
    else
      echo "Found OpenSSL version $OPENSSL_VERSION"
    fi
}

# NOTE: Each built archive must be a fat file, i.e support all required architectures
# Can be checked via e.g. `lipo -info $OUTPUT_DIR/OpenSSL-for-iPhone/lib/libssl.a`
build_crypto() {
    if [ ! -d $OUTPUT_DIR/OpenSSL-for-iPhone ]; then
        git clone https://github.com/x2on/OpenSSL-for-iPhone.git $OUTPUT_DIR/OpenSSL-for-iPhone
    fi

    pushd $OUTPUT_DIR/OpenSSL-for-iPhone
        OPENSSL_VERSION_STRIPPED=$( echo "$OPENSSL_VERSION" | grep -Eo '[0-9]\.[0-9]\.[0-9][a-z]') # example: 1.1.1l_1a ---> 1.1.1l
        ./build-libssl.sh --version="$OPENSSL_VERSION_STRIPPED"
    popd
}

build_libsodium() {
    if [ ! -d $OUTPUT_DIR/libsodium-ios ]; then
        git clone https://github.com/evernym/libsodium-ios.git $OUTPUT_DIR/libsodium-ios
    fi

    pushd $OUTPUT_DIR/libsodium-ios
        ./libsodium.rb
    popd
}

build_libzmq() {
    if [ ! -d $OUTPUT_DIR/libzmq-ios ]; then
        git clone https://github.com/evernym/libzmq-ios.git $OUTPUT_DIR/libzmq-ios
    fi

    pushd $OUTPUT_DIR/libzmq-ios
        git apply $SCRIPT_DIR/patches/libzmq.rb.patch
        ./libzmq.rb
    popd
}

# NOTE: $OUTPUT_DIR/libs/{arm64,x86_64}/$LIB_NAME.a should be a non-fat file with arm64 / x86_64 architecture
extract_architectures() {
    ARCHS="arm64 x86_64"
    FILE_PATH=$1
    LIB_FILE_NAME=$2
    LIB_NAME=$3

    echo FILE_PATH=$FILE_PATH
    echo LIB_FILE_NAME=$LIB_FILE_NAME

    mkdir -p $OUTPUT_DIR/libs
    pushd $OUTPUT_DIR/libs
        echo "Extracting architectures for $LIB_FILE_NAME..."
        for ARCH in ${ARCHS[*]}; do
            DESTINATION=${LIB_NAME}/${ARCH}

            echo "Destination $DESTINATION"

            mkdir -p $DESTINATION
            lipo -extract ${ARCH} $FILE_PATH -o $DESTINATION/$LIB_FILE_NAME-fat.a
            lipo $DESTINATION/$LIB_FILE_NAME-fat.a -thin $ARCH -output $DESTINATION/$LIB_FILE_NAME.a
            rm $DESTINATION/$LIB_FILE_NAME-fat.a
        done
    popd
}

checkout_indy_sdk() {
    if [ ! -d $INDY_SDK_DIR ]; then
        git clone https://github.com/hyperledger/indy-sdk $INDY_SDK_DIR
    fi

    pushd $INDY_SDK_DIR
        git fetch --all
        git checkout $INDY_VERSION
    popd
}

# NOTE: $INDY_SDK_DIR/libindy/target/$TRIPLET/release/libindy.a should be a non-fat file
build_libindy() {
    # OpenSSL-for-iPhone currently provides libs only for aarch64-apple-ios and x86_64-apple-ios, so we select only them.
    TRIPLETS="aarch64-apple-ios,x86_64-apple-ios"

    pushd $INDY_SDK_DIR/libindy
        cargo lipo --release --targets="${TRIPLETS}"
    popd
}

copy_libindy_architectures() {
    ARCHS="arm64 x86_64"
    LIB_NAME="indy"

    echo "Copying architectures for $LIB_NAME..."
    for ARCH in ${ARCHS[*]}; do
        generate_flags $ARCH

        echo ARCH=$ARCH
        echo TRIPLET=$TRIPLET

        mkdir -p $OUTPUT_DIR/libs/$LIB_NAME/$ARCH
        cp -v $INDY_SDK_DIR/libindy/target/$TRIPLET/release/libindy.a $OUTPUT_DIR/libs/$LIB_NAME/$ARCH/libindy.a
    done
}

# NOTE: $INDY_SDK_DIR/vcx/libvcx/target/$TRIPLET/release/libindy.a should be a non-fat file
build_libvcx() {
    WORK_DIR=$(abspath "$OUTPUT_DIR")
    ARCHS="arm64 x86_64"

    echo WORK_DIR=$WORK_DIR

    pushd $REPO_DIR/libvcx
        for ARCH in ${ARCHS[*]}; do
            generate_flags $ARCH

            echo ARCH=$ARCH
            echo TRIPLET=$TRIPLET

            export OPENSSL_LIB_DIR=$WORK_DIR/libs/openssl/${ARCH}
            export IOS_SODIUM_LIB=$WORK_DIR/libs/sodium/${ARCH}
            export IOS_ZMQ_LIB=$WORK_DIR/libs/zmq/${ARCH}
            export LIBINDY_DIR=$WORK_DIR/libs/indy/${ARCH}

            cargo build --target "${TRIPLET}" --release --no-default-features
        done
    popd
}

copy_libvcx_architectures() {
    ARCHS="arm64 x86_64"
    LIB_NAME="vcx"

    mkdir -p $OUTPUT_DIR/libs

    echo "Copying architectures for $LIB_NAME..."
    for ARCH in ${ARCHS[*]}; do
        generate_flags $ARCH

        echo ARCH=$ARCH
        echo TRIPLET=$TRIPLET

        mkdir -p $OUTPUT_DIR/libs/$LIB_NAME/$ARCH

        cp -v $REPO_DIR/target/$TRIPLET/release/libvcx.a $OUTPUT_DIR/libs/$LIB_NAME/$ARCH/libvcx.a
    done
}

copy_libs_to_combine() {
    mkdir -p $OUTPUT_DIR/cache/arch_libs

    copy_lib_tocombine sodium libsodium
    copy_lib_tocombine zmq libzmq
    copy_lib_tocombine vcx libvcx
}

copy_lib_tocombine() {
    LIB_NAME=$1
    LIB_FILE_NAME=$2

    ARCHS="arm64 x86_64"

    for ARCH in ${ARCHS[*]}; do
        cp -v "$OUTPUT_DIR/libs/$LIB_NAME/$ARCH/$LIB_FILE_NAME.a" "$OUTPUT_DIR/cache/arch_libs/${LIB_FILE_NAME}_$ARCH.a"
    done
}

combine_libs() {
    COMBINED_LIB=$1

    BUILD_CACHE=$(abspath "$OUTPUT_DIR/cache")

    ARCHS="arm64 x86_64"
    combined_libs_paths=""
    for arch in ${ARCHS[*]}; do
        libraries="libsodium libzmq libvcx" # libssl libcrypto libindy were already statically linked into libvcx during its build (see libvcx/build.rs)

        libs_to_combine_paths=""
        for library in ${libraries[*]}; do
          libs_to_combine_paths="${libs_to_combine_paths} ${BUILD_CACHE}/arch_libs/${library}_${arch}.a"
        done

        COMBINED_LIB_PATH=${BUILD_CACHE}/arch_libs/${COMBINED_LIB}_${arch}.a
        echo "Going to combine following libraries: '${libs_to_combine_paths}' to create combined library: '$COMBINED_LIB_PATH'"
        rm -rf "$COMBINED_LIB_PATH"
        libtool -static ${libs_to_combine_paths} -o "$COMBINED_LIB_PATH"
        combined_libs_paths="${combined_libs_paths} $COMBINED_LIB_PATH"
    done

    for arch in ${ARCHS[*]}; do
        COMBINED_LIB_PATH=${BUILD_CACHE}/arch_libs/${COMBINED_LIB}_${arch}.a
        echo "Lipo info about combined library ${COMBINED_LIB_PATH}:"
        lipo -info "$COMBINED_LIB_PATH"
    done

    FAT_COMBINED_LIB_PATH="$OUTPUT_DIR/${COMBINED_LIB}.a"
    echo "Using combined_libs_paths: ${combined_libs_paths} to combine them into single fat library: ${FAT_COMBINED_LIB_PATH}"
    lipo -create ${combined_libs_paths} -o "${FAT_COMBINED_LIB_PATH}"

    echo "Lipo info about combined library ${FAT_COMBINED_LIB_PATH}:"
    lipo -info "${FAT_COMBINED_LIB_PATH}"
}

build_vcx_framework() {
    COMBINED_LIB=$1
    ARCHS="arm64 x86_64"

    cp -v $OUTPUT_DIR/${COMBINED_LIB}.a $REPO_DIR/wrappers/ios/vcx/lib/libvcx.a

    pushd $REPO_DIR/wrappers/ios/vcx
        rm -rf vcx.framework.previousbuild

        for ARCH in ${ARCHS[*]}; do
            echo "Building vcx framework for $ARCH architecture"

            rm -rf vcx.framework
            if [ "${ARCH}" = "i386" ] || [ "${ARCH}" = "x86_64" ]; then
                # This sdk supports i386 and x86_64
                IPHONE_SDK=iphonesimulator
            elif [ "${ARCH}" = "armv7" ] || [ "${ARCH}" = "armv7s" ] || [ "${ARCH}" = "arm64" ]; then
                # This sdk supports armv7, armv7s, and arm64
                IPHONE_SDK=iphoneos
            else
                echo "Missing IPHONE_SDK value!"
                exit 1
            fi

            xcodebuild -project vcx.xcodeproj -scheme vcx -configuration Release -arch ${ARCH} -sdk ${IPHONE_SDK} CONFIGURATION_BUILD_DIR=. build

            if [ -d "./vcx.framework.previousbuild" ]; then
                lipo -create -output combined.ios.vcx vcx.framework/vcx vcx.framework.previousbuild/vcx
                mv combined.ios.vcx vcx.framework/vcx
                rm -rf vcx.framework.previousbuild
            fi
            cp -rp vcx.framework vcx.framework.previousbuild
        done

        rm lib/libvcx.a
        rm -rf vcx.framework.previousbuild
        mkdir -p vcx.framework/Headers
        cp -v ConnectMeVcx.h vcx.framework/Headers
        cp -v include/libvcx.h vcx.framework/Headers
        cp -v vcx/vcx.h vcx.framework/Headers
        cp -v utils/*.h vcx.framework/Headers
        if [ -d tmp ]; then
            rm -rf tmp
        fi

        UNIVERSAL_BUILD_PATH=$OUTPUT_DIR/universal/vcx
        mkdir -p $UNIVERSAL_BUILD_PATH
        cp -rvp vcx.framework $UNIVERSAL_BUILD_PATH
        pushd $UNIVERSAL_BUILD_PATH
            zip -r $OUTPUT_DIR/libvcx-ios-${LIBVCX_VERSION}-universal.zip ./*
        popd

        DEVICE_BUILD_PATH=$OUTPUT_DIR/device/vcx
        mkdir -p $DEVICE_BUILD_PATH
        cp -rvp vcx.framework $DEVICE_BUILD_PATH
        lipo -extract arm64 $DEVICE_BUILD_PATH/vcx.framework/vcx -o $DEVICE_BUILD_PATH/vcx.framework/vcx
        pushd $DEVICE_BUILD_PATH
            zip -r $OUTPUT_DIR/libvcx-ios-${LIBVCX_VERSION}-device.zip ./*
        popd

    popd
}

generate_flags() {
    if [ -z $1 ]; then
        echo "please provide the arch e.g. arm64 or x86_64"
        exit 1
    fi

    if [ $1 == "arm64" ]; then
        export TRIPLET="aarch64-apple-ios"
    elif [ $1 == "x86_64" ]; then
        export TRIPLET="x86_64-apple-ios"
    fi
}

abspath() {
    # generate absolute path from relative path
    # $1     : relative filename
    # return : absolute path
    if [ -d "$1" ]; then
        # dir
        (
            cd "$1"
            pwd
        )
    elif [ -f "$1" ]; then
        # file
        if [[ $1 = /* ]]; then
            echo "$1"
        elif [[ $1 == */* ]]; then
            echo "$(
                cd "${1%/*}"
                pwd
            )/${1##*/}"
        else
            echo "$(pwd)/$1"
        fi
    fi
}

# Setup environment
setup

# Build 3rd party libraries
build_crypto
build_libsodium
build_libzmq

# Extract architectures from fat files into non-fat files
extract_architectures $OUTPUT_DIR/libsodium-ios/dist/ios/lib/libsodium.a libsodium sodium
extract_architectures $OUTPUT_DIR/libzmq-ios/dist/ios/lib/libzmq.a libzmq zmq
extract_architectures $OUTPUT_DIR/OpenSSL-for-iPhone/lib/libssl.a libssl openssl
extract_architectures $OUTPUT_DIR/OpenSSL-for-iPhone/lib/libcrypto.a libcrypto openssl

# Build libindy
checkout_indy_sdk
build_libindy
copy_libindy_architectures

# Build vcx
build_libvcx
copy_libvcx_architectures

# Copy libraries to combine

tree "$OUTPUT_DIR/libs/"
copy_libs_to_combine
tree "$OUTPUT_DIR/cache/arch_libs"

# Combine libs by arch and merge libs to single fat binary
combine_libs libvcx_all

# Build vcx.framework
build_vcx_framework libvcx_all
