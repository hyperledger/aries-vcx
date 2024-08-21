    #!/bin/bash
    set -ex

    SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

    # Required env vars
    ARIES_VCX_ROOT=$(dirname $(dirname $(dirname $(dirname $SCRIPT_DIR))))
    IOS_BUILD_DEPS_DIR=${ARIES_VCX_ROOT}/target/ios_build_deps
    LANGUAGE="swift"
    TARGET="aarch64-apple-ios"
    TARGET_NICKNAME="arm64"
    ABI="iphoneos"

    generate_bindings() {
        export UNIFFI_ROOT="${ARIES_VCX_ROOT}/aries/wrappers/uniffi-aries-vcx"
        export IOS_APP_DIR="${ARIES_VCX_ROOT}/aries/agents/ios/ariesvcx/ariesvcx"

        pushd "${UNIFFI_ROOT}/core"
            cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/vcx.udl --language ${LANGUAGE}
        popd
        
        cp -R ${UNIFFI_ROOT}/core/src/vcx.swift ${UNIFFI_ROOT}/core/src/vcxFFI.* ${IOS_APP_DIR}
        rm -R ${UNIFFI_ROOT}/core/src/vcx.swift ${UNIFFI_ROOT}/core/src/vcxFFI.*
    }

    build_uniffi_for_demo() {
        echo "Running build_uniffi_for_demo..."
        export UNIFFI_ROOT="${ARIES_VCX_ROOT}/aries/wrappers/uniffi-aries-vcx"
        export IOS_APP_DIR="${ARIES_VCX_ROOT}/aries/agents/ios/ariesvcx/ariesvcx"
        export ABI_PATH=${IOS_APP_DIR}/Frameworks
        mkdir -p ${ABI_PATH}

        pushd ${UNIFFI_ROOT}/core
            cargo build --target ${TARGET}
            cp ${ARIES_VCX_ROOT}/target/${TARGET}/debug/libuniffi_vcx.a ${ABI_PATH}/libuniffi_vcx.a

        popd
    }

    generate_bindings
    build_uniffi_for_demo
