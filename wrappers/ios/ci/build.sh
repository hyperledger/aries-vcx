#!/bin/sh

set -ex

export PKG_CONFIG_ALLOW_CROSS=1

OPENSSL_VERSION="1.1.1t"

REPO_DIR=$PWD
SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"
OUTPUT_DIR=/tmp/artifacts

setup() {
    echo "ios/ci/build.sh: running setup()"

    echo "Setup rustup"
    rustup default 1.65.0
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
    which automake &>/dev/null || brew install automake
    which autoconf &>/dev/null || brew install autoconf
    which cmake &>/dev/null || brew install cmake

    mkdir -p $OUTPUT_DIR/libs
    mkdir -p $OUTPUT_DIR/arch_libs
}

build_crypto() {
    echo "ios/ci/build.sh: running build_crypto()"

    if [ ! -d $OUTPUT_DIR/OpenSSL-for-iPhone ]; then
        git clone https://github.com/x2on/OpenSSL-for-iPhone.git $OUTPUT_DIR/OpenSSL-for-iPhone

        # Need to use an older version of the build script to support older iOS versions
        pushd $OUTPUT_DIR/OpenSSL-for-iPhone
            git checkout b77ace70b2594de69c88d0748326d2a1190bbac1
        popd
    fi

    pushd $OUTPUT_DIR/OpenSSL-for-iPhone
        ./build-libssl.sh --version="$OPENSSL_VERSION" --targets="ios64-cross-arm64 ios-sim-cross-x86_64"
        mv lib/libssl.a lib/libssl.a.fat
        mv lib/libcrypto.a lib/libcrypto.a.fat
    popd

    export OPENSSL_INCLUDE_DIR="$OUTPUT_DIR/OpenSSL-for-iPhone/include"
}

extract_crypto_lib() {
    echo "ios/ci/build.sh: running extract_crypto_lib()"

    ARCH=$1
    LIBS="libcrypto libssl"
    echo "ios/ci/build.sh: running extract_crypto_lib()"

    for LIB in ${LIBS[*]}; do
        pushd $OUTPUT_DIR/OpenSSL-for-iPhone/lib
            rm -f ${LIB}.a
            lipo ${LIB}.a.fat -thin ${ARCH} -output ${LIB}.a
        popd
    done
}

build_libsodium() {
    echo "ios/ci/build.sh: running build_libsodium()"

    if [ ! -d "$OUTPUT_DIR/libsodium-ios" ]; then
        mkdir $OUTPUT_DIR/libsodium-ios
    fi

    pushd $OUTPUT_DIR/libsodium-ios
        $REPO_DIR/wrappers/ios/ci/build_libsodium.sh "$OUTPUT_DIR/libsodium-ios" $1 $OUTPUT_DIR/libs
    popd
}

build_libzmq() {
    echo "ios/ci/build.sh: running build_libzmq()"

    if [ ! -d "$OUTPUT_DIR/libzmq-ios" ]; then
        mkdir $OUTPUT_DIR/libzmq-ios
    fi

    pushd $OUTPUT_DIR/libzmq-ios
        $REPO_DIR/wrappers/ios/ci/build_libzmq.sh "$OUTPUT_DIR/libzmq-ios" $1 $OUTPUT_DIR/libs $OUTPUT_DIR/libsodium-ios
    popd
}

build_libvcx() {
    echo "ios/ci/build.sh: running build_libvcx()"

    pushd $REPO_DIR/libvcx
        ARCH=$1
        TRIPLET=$2

        export OPENSSL_LIB_DIR="$OUTPUT_DIR/OpenSSL-for-iPhone/lib"
        export PKG_CONFIG_PATH="$OUTPUT_DIR/libsodium-ios/build/${ARCH}/lib/pkgconfig:$OUTPUT_DIR/libzmq-ios/build/${ARCH}/lib/pkgconfig"

        cargo build --target "${TRIPLET}" --release

        rm -f "$OUTPUT_DIR/libs/libvcx.a"
        ln -s "$REPO_DIR/target/$TRIPLET/release/libvcx.a" "$OUTPUT_DIR/libs/libvcx.a"
    popd
}

combine_static_libs() {
    echo "ios/ci/build.sh: running combine_static_libs()"

    COMBINED_LIB=$1
    ARCH=$2
    combined_libs_paths=""

    libraries="libsodium libzmq libvcx" # libssl, libcrypto, libindy were statically linked into libvcx during its build (see libvcx/build.rs)
    libs_to_combine_paths=""

    for library in ${libraries[*]}; do
        libs_to_combine_paths="${libs_to_combine_paths} "$OUTPUT_DIR/libs/${library}".a"
    done

    COMBINED_LIB_PATH=${OUTPUT_DIR}/arch_libs/${COMBINED_LIB}_${ARCH}.a
    echo "Going to combine following libraries: '${libs_to_combine_paths}' to create combined library: '$COMBINED_LIB_PATH'"
    libtool -static ${libs_to_combine_paths} -o "$COMBINED_LIB_PATH"
}

make_fat_library() {
    echo "ios/ci/build.sh: running make_fat_library()"

    COMBINED_LIB=$1
    ARCHS="arm64 x86_64"
    COMBINED_LIB_PATHS=""

    for arch in ${ARCHS[*]}; do
        COMBINED_LIB_PATH=${OUTPUT_DIR}/arch_libs/${COMBINED_LIB}_${arch}.a
        COMBINED_LIB_PATHS="${COMBINED_LIB_PATHS} $COMBINED_LIB_PATH"
        echo "Lipo info about combined library ${COMBINED_LIB_PATH}:"
        lipo -info "$COMBINED_LIB_PATH"
    done

    FAT_COMBINED_LIB_PATH="$OUTPUT_DIR/${COMBINED_LIB}.a"
    echo "Using combined_libs_paths: ${COMBINED_LIB_PATHS} to combine them into single fat library: ${FAT_COMBINED_LIB_PATH}"
    lipo -create ${COMBINED_LIB_PATHS} -o "${FAT_COMBINED_LIB_PATH}"

    echo "Lipo info about combined library ${FAT_COMBINED_LIB_PATH}:"
    lipo -info "${FAT_COMBINED_LIB_PATH}"
}

build_vcx_framework() {
    echo "ios/ci/build.sh: running build_vcx_framework() COMBINED_LIB=${COMBINED_LIB}"

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
        cp -v VcxAPI.h vcx.framework/Headers
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

# Initial setup and OpenSSL building
setup
build_crypto 

########### iOS ARM64 ###########
extract_crypto_lib "arm64"
build_libsodium "arm64"
build_libzmq "arm64"
build_libvcx "arm64" "aarch64-apple-ios"
combine_static_libs "libvcx_all" "arm64"

########### iOS x86_64 Simulator ###########
extract_crypto_lib "x86_64"
build_libsodium "x86_64"
build_libzmq "x86_64"
build_libvcx "x86_64" "x86_64-apple-ios"
combine_static_libs "libvcx_all" "x86_64"

# Combine each arch libvcx lib into one fat lib
make_fat_library "libvcx_all"

# Build Xcode framework
build_vcx_framework "libvcx_all"