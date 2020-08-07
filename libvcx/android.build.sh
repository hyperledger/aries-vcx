#!/usr/bin/env bash

# TODO: Move this to wrapper folder

set -ex

export LIBVCX_WORKDIR="$( cd "$(dirname "$0")" ; pwd -P )"
export ANDROID_BUILD_FOLDER="/tmp/android_build"
JAVA_WRAPPER_DIR="${LIBVCX_WORKDIR}/../wrappers/java"

TARGET_ARCHS="$@"

source $LIBVCX_WORKDIR/../ci/scripts/setup.android.env.sh

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

    # TODO: Get and copy includes
    cp ${LIBVCX_WORKDIR}/target/${TRIPLET}/release/{libvcx.a,libvcx.so} ${PACKAGE_DIR}/lib

    if [ -z "${LIBVCX_VERSION}" ]; then
        zip -r ${ZIP_DIR}/libvcx_android_${ABSOLUTE_ARCH}.zip ${PACKAGE_DIR}
    else
        zip -r ${ZIP_DIR}/libvcx_android_${ABSOLUTE_ARCH}_${LIBVCX_VERSION}.zip ${PACKAGE_DIR}
    fi
    cp $(ls -r -t1 ${JAVA_WRAPPER_DIR}/android/build/outputs/aar/* | head -n 1) ${AAR_DIR}
}

build_android_wrapper(){
    pushd ${JAVA_WRAPPER_DIR}
        pushd android
            npm install
        popd

        ./gradlew --no-daemon clean build --project-dir=android -x test
    popd
}


for TARGET_ARCH in ${TARGET_ARCHS}
do
    generate_arch_flags ${TARGET_ARCH}
    setup_dependencies_env_vars ${TARGET_ARCH}
    set_env_vars

    create_standalone_toolchain_and_rust_target
    create_cargo_config

    build_libvcx

    copy_libraries_to_jni ${JAVA_WRAPPER_DIR} ${TARGET_ARCH}
done

accept_licenses
build_android_wrapper

prepare_artifacts
