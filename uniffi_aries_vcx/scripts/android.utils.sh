#!/usr/bin/env bash
export BLACK=`tput setaf 0`
export RED=`tput setaf 1`
export GREEN=`tput setaf 2`
export YELLOW=`tput setaf 3`
export BLUE=`tput setaf 4`
export MAGENTA=`tput setaf 5`
export CYAN=`tput setaf 6`
export WHITE=`tput setaf 7`
export BOLD=`tput bold`
export RESET=`tput sgr0`

if [ -z "${ANDROID_BUILD_DIR}" ]; then
    echo STDERR "${RED}ANDROID_BUILD_DIR is not set. Please set it in the caller script${RESET}"
    exit 1
fi

create_cargo_config(){
set_android_arch_env
mkdir -p ${HOME}/.cargo
cat << EOF > ${HOME}/.cargo/config
[target.${TRIPLET}]
ar = "$(realpath ${AR})"
linker = "$(realpath ${CC})"
EOF
}

create_standalone_toolchain_and_rust_target() {
    generate_arch_flags ${1}
    # will only create toolchain if not already created
    python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py \
        --arch ${TARGET_ARCH} \
        --api ${TARGET_API} \
        --stl=libc++ \
        --force \
        --install-dir ${TOOLCHAIN_DIR}

    # add rust target
    rustup target add ${TRIPLET}
}

download_and_setup_toolchain() {
    if [ "$(uname)" == "Darwin" ]; then
        export TOOLCHAIN_PREFIX="${ANDROID_BUILD_DIR}/toolchains/darwin"
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        echo "${GREEN}Resolving NDK for OSX${RESET}"
        download_and_unzip_if_missed "android-ndk-r20" "https://dl.google.com/android/repository/android-ndk-r20-darwin-x86_64.zip"
        popd
    elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
        export TOOLCHAIN_PREFIX="${ANDROID_BUILD_DIR}/toolchains/linux"
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        echo "${GREEN}Resolving NDK for Linux${RESET}"
        download_and_unzip_if_missed "android-ndk-r20" "https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip"
        popd
    fi
    export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r20
    echo "${GREEN}NDK RESOLVED AT${RESET} ${ANDROID_NDK_ROOT}"
}

download_and_unzip_if_missed() {
    expected_directory="$1"
    url="$2"
    fname="tmp_$(date +%s)_$expected_directory.zip"
    if [ ! -d "${expected_directory}" ] ; then
        echo "Downloading ${GREEN}${url}${RESET} as ${GREEN}${fname}${RESET}"
        wget -q -O ${fname} "${url}"
        echo "Unzipping ${GREEN}${fname}${RESET}"
        unzip -qqo "${fname}"
        rm "${fname}"
        echo "${GREEN}Done!${RESET}"
    else
        echo "${BLUE}Skipping download ${url}${RESET}. Expected directory ${expected_directory} was found"
    fi
}

download_sdk() {
    mkdir -p ${ANDROID_SDK}
    pushd ${ANDROID_SDK}
        download_and_unzip_if_missed "tools" "https://dl.google.com/android/repository/commandlinetools-linux-10406996_latest.zip"
        mv cmdline-tools tools
    popd
}

generate_arch_flags() {
    if [ -z $1 ]; then
        echo STDERR "${RED}Please provide the arch e.g arm, armv7, x86 or arm64${RESET}"
        exit 1
    fi
    export TARGET_ARCH=$1
    download_and_setup_toolchain

    if [ $1 == "arm" ]; then
        export TARGET_API="30"
        export TRIPLET="arm-linux-androideabi"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "armv7" ]; then
        export TARGET_ARCH="arm"
        export TARGET_API="30"
        export TRIPLET="armv7-linux-androideabi"
        export ANDROID_TRIPLET="arm-linux-androideabi"
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "arm64" ]; then
        export TARGET_API="30"
        export TRIPLET="aarch64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="arm64-v8a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86" ]; then
        export TARGET_API="30"
        export TRIPLET="i686-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86_64" ]; then
        export TARGET_API="30"
        export TRIPLET="x86_64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86_64"
        export TOOLCHAIN_SYSROOT_LIB="lib64"
    fi

    export TOOLCHAIN_DIR=${TOOLCHAIN_PREFIX}/${TARGET_ARCH}
    export PATH=${TOOLCHAIN_DIR}/bin:${PATH}
}

prepare_dependencies() {
    if [ -z $1 ]; then
        echo STDERR "${RED}Please provide the architecture e.g arm, armv7, x86, x86_64 or arm64.${RESET}"
        exit 1
    fi
    TARGET_ARCH="$1"
    echo "prepare_dependencies >> TARGET_ARCH=${TARGET_ARCH}"
    pushd "${ANDROID_BUILD_DIR}"
        download_and_unzip_if_missed "openssl_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps/openssl/openssl_$TARGET_ARCH.zip"
        download_and_unzip_if_missed "libsodium_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps/sodium/libsodium_$TARGET_ARCH.zip"
        download_and_unzip_if_missed "libzmq_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps/zmq/libzmq_$TARGET_ARCH.zip"
    popd
    set_dependencies_env_vars
    generate_arch_flags ${TARGET_ARCH}
    echo "${GREEN}SUCCESSFULLY PREPARED DEPS FOR${RESET} ${TARGET_ARCH}"
}

set_android_arch_env() {
    export CC=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang
    export AR=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ar
    export CXX=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang++
    export CXXLD=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ld
    export RANLIB=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ranlib
    export OBJCOPY=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-objcopy
}

set_android_env() {
    export PKG_CONFIG_ALLOW_CROSS=1
    export CARGO_INCREMENTAL=0
    export RUST_LOG=indy=trace
    export RUST_TEST_THREADS=1
    export RUST_BACKTRACE=1

    # SDK location
    export ANDROID_SDK=${ANDROID_BUILD_DIR}/sdk
    export PATH=${PATH}:${ANDROID_SDK}/platform-tools
    export PATH=${PATH}:${ANDROID_SDK}/tools
    export PATH=${PATH}:${ANDROID_SDK}/tools/bin

    export TARGET=android
}

set_dependencies_env_vars() {
    # main env vars that need to be set
    export OPENSSL_DIR=${ANDROID_BUILD_DIR}/openssl_${TARGET_ARCH}
    export SODIUM_DIR=${ANDROID_BUILD_DIR}/libsodium_${TARGET_ARCH}
    export LIBZMQ_DIR=${ANDROID_BUILD_DIR}/libzmq_${TARGET_ARCH}

    # export TOOLCHAIN_DIR=${TOOLCHAIN_PREFIX}/${TARGET_ARCH}
    # export PATH=${TOOLCHAIN_DIR}/bin:${PATH}

    # secondary env vars that need to be set
    export SODIUM_LIB_DIR=${SODIUM_DIR}/lib
    export SODIUM_INCLUDE_DIR=${SODIUM_DIR}/include
    export SODIUM_STATIC=1
    export LIBZMQ_LIB_DIR=${LIBZMQ_DIR}/lib
    export LIBZMQ_INCLUDE_DIR=${LIBZMQ_DIR}/include
    export LIBZMQ_PREFIX=${LIBZMQ_DIR}
    export OPENSSL_LIB_DIR=${OPENSSL_DIR}/lib
    export OPENSSL_STATIC=1
}

build_uniffi() {
    echo "**************************************************"
    echo "ARIES_VCX_ROOT is ${BOLD}${BLUE}${ARIES_VCX_ROOT}${RESET}"
    echo "Building for ${BOLD}${YELLOW}${TARGET_ARCH}${RESET}"
    echo "Toolchain path ${BOLD}${YELLOW}${TOOLCHAIN_DIR}${RESET}"
    echo "OpenSSL path ${BOLD}${YELLOW}${OPENSSL_DIR}${RESET}"
    echo "Sodium path ${BOLD}${YELLOW}${SODIUM_DIR}${RESET}"
    echo "ZMQ path ${BOLD}${YELLOW}${LIBZMQ_DIR}${RESET}"
    echo "**************************************************"
    export UNIFFI_ROOT="${ARIES_VCX_ROOT}/uniffi_aries_vcx"
    export ANDROID_DEMO_DIR="${UNIFFI_ROOT}/demo"
    export ABI_PATH=${ANDROID_DEMO_DIR}/app/src/main/jniLibs/${ABI}
    mkdir -p ${ABI_PATH}

    pushd "${UNIFFI_ROOT}/core"
        cargo build --lib --target=${TRIPLET}
        cp "$(realpath ${ARIES_VCX_ROOT}/target/${TRIPLET}/debug/libuniffi_vcx.so)" "$(realpath ${ABI_PATH}/libuniffi_vcx.so)"
        unset SODIUM_DIR OPENSSL_DIR LIBZMQ_DIR
        cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/vcx.udl --language ${LANGUAGE}
        cp -R "$(realpath ${UNIFFI_ROOT}/core/src/org/*)" "$(realpath ${ANDROID_DEMO_DIR}/app/src/main/java/org)"
        rm -R "$(realpath ${UNIFFI_ROOT}/core/src/org/)"
    popd
}
