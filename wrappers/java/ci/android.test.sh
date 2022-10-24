#!/usr/bin/env bash

set -ex

REPO_DIR=$PWD
SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"
LIBVCX_DIR="${REPO_DIR}/libvcx"

BUILD_TYPE="--release"
export ANDROID_BUILD_FOLDER="/tmp/android_build"

TARGET_ARCHS="$@"

source ${SCRIPT_DIR}/setup.android.env.sh

if [ -z "${TARGET_ARCHS}" ]; then
    echo STDERR "${RED}Missing TARGET_ARCHS argument${RESET}"
    echo STDERR "${BLUE}e.g. a list of archs such as arm, armv7, x86 or arm64${RESET}"
    exit 1
fi

source ${SCRIPT_DIR}/setup.android.env.sh

declare -a EXE_ARRAY

build_test_artifacts(){
    pushd ${LIBVCX_DIR}
        cargo clean

        SET_OF_TESTS='api_lib::api_c::wallet::tests::test_wallet_export_import'

        # This is needed to get the correct message if test are not
        # built. Next call will just reuse old results and parse the
        # response.
        cargo test ${BUILD_TYPE} --target=${TRIPLET} ${SET_OF_TESTS} \
                 --no-run --features general_test

        # Collect items to execute tests, uses resulting files from
        # previous step
        EXE_ARRAY=($(
            cargo test ${BUILD_TYPE} --target=${TRIPLET} ${SET_OF_TESTS} \
                     --features general_test --no-run --message-format=json \
                | jq -r "select(.profile.test == true) | .filenames[]"))

    popd
}

create_cargo_config(){
mkdir -p ${HOME}/.cargo
cat << EOF > ${HOME}/.cargo/config
[target.${TRIPLET}]
ar = "$(realpath ${AR})"
linker = "$(realpath ${CXX})"
EOF
}

execute_on_device(){

    set -x

    # adb -e push \
    # "${TOOLCHAIN_DIR}/sysroot/usr/lib/${ANDROID_TRIPLET}/libc++_shared.so" "/data/local/tmp/libc++_shared.so"

    # adb -e push \
    # "${SODIUM_LIB_DIR}/libsodium.so" "/data/local/tmp/libsodium.so"

    adb -e push \
    "${LIBZMQ_LIB_DIR}/libzmq.so" "/data/local/tmp/libzmq.so"

    adb -e logcat | grep indy &

    for i in "${EXE_ARRAY[@]}"
    do
       :
        EXE="${i}"
        EXE_NAME=`basename ${EXE}`

        adb -e push "$EXE" "/data/local/tmp/$EXE_NAME"
        adb -e shell "chmod 755 /data/local/tmp/$EXE_NAME"
        OUT="$(mktemp)"
        MARK="ADB_SUCCESS!"
        time adb -e shell "LD_LIBRARY_PATH=/data/local/tmp RUST_TEST_THREADS=1 RUST_BACKTRACE=full RUST_LOG=trace /data/local/tmp/$EXE_NAME && echo $MARK" 2>&1 | tee $OUT
        grep $MARK $OUT
    done

}

for TARGET_ARCH in ${TARGET_ARCHS}
do
    prepare_dependencies ${TARGET_ARCH}
    generate_arch_flags ${TARGET_ARCH}
    setup_dependencies_env_vars ${TARGET_ARCH}
    recreate_avd
    set_env_vars
    create_standalone_toolchain_and_rust_target
    create_cargo_config
    build_test_artifacts &&
    check_if_emulator_is_running &&
    execute_on_device
    kill_avd
done
