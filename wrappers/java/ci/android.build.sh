#!/usr/bin/env bash

set -ex

REPO_DIR=$PWD
SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"
LIBVCX_DIR="${REPO_DIR}"
JAVA_WRAPPER_DIR="${REPO_DIR}/wrappers/java"

TARGET_ARCHS="$@"
echo "android.build.sh >> TARGET_ARCHS=${TARGET_ARCHS}"


source ${SCRIPT_DIR}/setup.android.env.sh

if [ -z "${TARGET_ARCHS}" ]; then
    echo STDERR "${RED}Missing TARGET_ARCHS argument${RESET}"
    echo STDERR "${BLUE}e.g. a list of archs such as arm, armv7, x86 or arm64${RESET}"
    exit 1
fi

prepare_artifacts(){
    echo "${GREEN}Packaging library in zip file${RESET}"
    PACKAGE_DIR=${HOME}/artifacts/libvcx_${ABSOLUTE_ARCH}
    ZIP_DIR=${HOME}/artifacts/zip
    AAR_DIR=${HOME}/artifacts/aar
    mkdir -p ${PACKAGE_DIR}/{include,lib} ${ZIP_DIR} ${AAR_DIR}

    mv ${LIBVCX_DIR}/target/${TRIPLET}/release/libvcx.so ${PACKAGE_DIR}/lib
    rm -r ${LIBVCX_DIR}/target

    if [ -z "${LIBVCX_VERSION}" ]; then
        zip -r ${ZIP_DIR}/libvcx_android_${ABSOLUTE_ARCH}.zip ${PACKAGE_DIR}
        rm ${ZIP_DIR}/libvcx_android_${ABSOLUTE_ARCH}.zip
    else
        zip -r ${ZIP_DIR}/libvcx_android_${ABSOLUTE_ARCH}_${LIBVCX_VERSION}.zip ${PACKAGE_DIR}
        rm ${ZIP_DIR}/libvcx_android_${ABSOLUTE_ARCH}_${LIBVCX_VERSION}.zip
    fi
    mv $(ls -r -t1 ${JAVA_WRAPPER_DIR}/android/build/outputs/aar/* | head -n 1) ${AAR_DIR}
    rm -r ${JAVA_WRAPPER_DIR}/android/
}

build_android_wrapper(){
    pushd ${JAVA_WRAPPER_DIR}
        ./gradlew --no-daemon clean build --project-dir=android
    popd
}


for TARGET_ARCH in ${TARGET_ARCHS}
do
    prepare_dependencies ${TARGET_ARCH}
    generate_arch_flags ${TARGET_ARCH}
    setup_dependencies_env_vars ${TARGET_ARCH}
    set_env_vars

    create_standalone_toolchain_and_rust_target
    create_cargo_config

    # The prebuild libzmq.so exports not only symbols zmq_* but many
    # others. Some of these symbols conflicts symbols from libsodium.
    #
    # Hide all !^zmq_ symbols
    #
    _libzmq=${LIBZMQ_DIR}/lib/libzmq.so
    for s in $(nm -g ${_libzmq} | egrep ' (T|V) ' | awk '!/ T zmq/ { print $3 }'); do
        cp ${_libzmq} ${_libzmq}.in
        ${OBJCOPY} -L ${s} ${_libzmq}.in ${_libzmq}
    done
    rm ${_libzmq}.in

    build_libvcx ${LIBVCX_DIR}

    copy_libraries_to_jni ${JAVA_WRAPPER_DIR} ${TARGET_ARCH} ${LIBVCX_DIR}
done

accept_licenses
build_android_wrapper

prepare_artifacts
