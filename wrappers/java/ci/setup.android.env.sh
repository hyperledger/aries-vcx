#!/usr/bin/env bash
set -x
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

if [ -z "${ANDROID_BUILD_FOLDER}" ]; then
    echo STDERR "ANDROID_BUILD_FOLDER is not set. Please set it in the caller script"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi

ANDROID_SDK=${ANDROID_BUILD_FOLDER}/sdk
export ANDROID_SDK_ROOT=${ANDROID_SDK}
export ANDROID_HOME=${ANDROID_SDK}
export PATH=${PATH}:${ANDROID_HOME}/platform-tools
export PATH=${PATH}:${ANDROID_HOME}/tools
export PATH=${PATH}:${ANDROID_HOME}/tools/bin

mkdir -p ${ANDROID_SDK}

TARGET_ARCH=$1


delete_existing_avd(){
    kill_avd
    avdmanager delete avd -n ${ABSOLUTE_ARCH}
}

accept_licenses(){
    yes | sdkmanager --licenses
}

# TODO: Recreating avd for more than a single arch doesn't work
create_avd(){
    echo "${GREEN}Creating Android SDK${RESET}"

    accept_licenses

    if [ ! -d "${ANDROID_SDK}/emulator/" ] ; then
        echo "yes" |
              sdkmanager --no_https \
                "emulator" \
                "platform-tools" \
                "platforms;android-24" \
                "system-images;android-24;default;${ABI}" > sdkmanager.install.emulator.and.tools.out 2>&1

        # TODO sdkmanager upgrades by default. Hack to downgrade Android Emulator so as to work in headless mode (node display).
        # Remove as soon as headless mode is fixed.
        mv /home/indy/emu.zip emu.zip
        mv emulator emulator_backup
        unzip emu.zip
        rm "emu.zip"
    else
        echo "Skipping sdkmanager activity"
    fi

    echo "${BLUE}Creating android emulator${RESET}"

    echo "no" |
         avdmanager -v create avd \
            --name ${ABSOLUTE_ARCH} \
            --package "system-images;android-24;default;${ABI}" \
            -f \
            -c 3000M
    ANDROID_SDK_ROOT=${ANDROID_SDK} ANDROID_HOME=${ANDROID_SDK} ${ANDROID_HOME}/tools/emulator -avd ${ABSOLUTE_ARCH} -netdelay none -partition-size 3000 -netspeed full -no-audio -no-window -no-snapshot -no-accel &
}

kill_avd(){
    adb devices | grep emulator | cut -f1 | while read line; do adb -s $line emu kill; done || true
}

recreate_avd(){
    pushd ${ANDROID_SDK}
        set +e
        delete_existing_avd
        set -e
        create_avd
    popd
}

check_if_emulator_is_running(){
    tries=0
    running=false
    emus=$(adb devices)
    while [ $running = false ]
    do
      if [ $tries -gt 5 ]; then
        echo 'Exceeded the number of attempts to check the emulator status, shutting down'
        exit 1
      else
        sleep 30
      fi
      if [[ ${emus} = *"emulator"* ]]; then
          echo "emulator is running"
          running=true
          until adb -e shell "ls /storage/emulated/0/"
          do
              echo "waiting for emulator FS"
              sleep 30
          done
      else
          echo "Emulator is not running, tried $[$tries+1] times"
      fi
      tries=$[$tries+1]
    done
}

create_cargo_config(){
mkdir -p ${HOME}/.cargo
cat << EOF > ${HOME}/.cargo/config
[target.${TRIPLET}]
ar = "$(realpath ${AR})"
linker = "$(realpath ${CC})"
EOF
}

download_emulator() {
    curl -o /home/indy/emu.zip https://dl.google.com/android/repository/emulator-linux-5889189.zip
}

download_and_unzip_if_missed() {
    target_dir=$1
    url_pref=$2
    fname=$3
    url="${url_pref}${fname}"
    if [ ! -d "${target_dir}" ] ; then
        echo "${GREEN}Downloading ${fname}${RESET} from url ${url}"
        wget -q ${url}
        unzip -qqo ${fname}
        rm ${fname}
        echo "${GREEN}Done!${RESET}"
    else
        echo "${BLUE}Skipping download ${fname}${RESET}"
    fi
}

download_and_unzip_if_missed_1() {
    expected_directory="$1"
    url="$2"
    fname="tmp_$(date +%s)_expected_directory.zip"
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

download_sdk(){
    pushd ${ANDROID_SDK}
        download_and_unzip_if_missed "tools" "https://dl.google.com/android/repository/" "sdk-tools-linux-4333796.zip"
    popd
}

generate_arch_flags(){
    if [ -z $1 ]; then
        echo STDERR "${RED}Please provide the arch e.g arm, armv7, x86 or arm64${RESET}"
        exit 1
    fi
    export ABSOLUTE_ARCH=$1
    export TARGET_ARCH=$1
    if [ $1 == "arm" ]; then
        export TARGET_API="24"
        export TRIPLET="arm-linux-androideabi"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "armv7" ]; then
        export TARGET_ARCH="arm"
        export TARGET_API="24"
        export TRIPLET="armv7-linux-androideabi"
        export ANDROID_TRIPLET="arm-linux-androideabi"
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "arm64" ]; then
        export TARGET_API="24"
        export TRIPLET="aarch64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="arm64-v8a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86" ]; then
        export TARGET_API="24"
        export TRIPLET="i686-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86_64" ]; then
        export TARGET_API="24"
        export TRIPLET="x86_64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86_64"
        export TOOLCHAIN_SYSROOT_LIB="lib64"
    fi

}

prepare_dependencies() {
    TARGET_ARCH="$1"
    echo "prepare_dependencies >> TARGET_ARCH=${TARGET_ARCH}"
    pushd "${ANDROID_BUILD_FOLDER}"
        download_and_unzip_if_missed_1 "openssl_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps-libc++/openssl/openssl_$TARGET_ARCH.zip"
        download_and_unzip_if_missed_1 "libsodium_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps-libc++/sodium/libsodium_$TARGET_ARCH.zip"
        download_and_unzip_if_missed_1 "libzmq_$TARGET_ARCH" "https://repo.sovrin.org/android/libindy/deps-libc++/zmq/libzmq_$TARGET_ARCH.zip"

        if [ "$TARGET_ARCH" == "arm" ]; then
          download_and_unzip_if_missed_1 "libindy_arm" "https://gitlab.com/evernym/verity/vdr-tools/-/package_files/16384051/download"
        elif [ "$TARGET_ARCH" == "arm64" ]; then
          download_and_unzip_if_missed_1 "libindy_arm64" "https://gitlab.com/evernym/verity/vdr-tools/-/package_files/16384046/download"
        elif [ "$TARGET_ARCH" == "armv7" ]; then
          download_and_unzip_if_missed_1 "libindy_armv7" "https://gitlab.com/evernym/verity/vdr-tools/-/package_files/16384064/download"
        elif [ "$TARGET_ARCH" == "x86_64" ]; then
          download_and_unzip_if_missed_1 "libindy_x86_64" "https://gitlab.com/evernym/verity/vdr-tools/-/package_files/16384056/download"
        elif [ "$TARGET_ARCH" == "x86" ]; then
          download_and_unzip_if_missed_1 "libindy_x86" "https://gitlab.com/evernym/verity/vdr-tools/-/package_files/16384034/download"
        fi
    popd
}

setup_dependencies_env_vars(){
    export OPENSSL_DIR=${ANDROID_BUILD_FOLDER}/openssl_$1
    export SODIUM_DIR=${ANDROID_BUILD_FOLDER}/libsodium_$1
    export LIBZMQ_DIR=${ANDROID_BUILD_FOLDER}/libzmq_$1
    export INDY_DIR=${ANDROID_BUILD_FOLDER}/libindy_$1
}

create_standalone_toolchain_and_rust_target(){
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

download_and_setup_toolchain(){
    if [ "$(uname)" == "Darwin" ]; then
        export TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/darwin
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        echo "${GREEN}Resolving NDK for OSX${RESET}"
        download_and_unzip_if_missed "android-ndk-r20" "https://dl.google.com/android/repository/" "android-ndk-r20-darwin-x86_64.zip"
        popd
    elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
        export TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/linux
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        echo "${GREEN}Resolving NDK for Linux${RESET}"
        download_and_unzip_if_missed "android-ndk-r20" "https://dl.google.com/android/repository/" "android-ndk-r20-linux-x86_64.zip"
        popd
    fi
    export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r20
}

set_env_vars(){
    export PKG_CONFIG_ALLOW_CROSS=1
    export CARGO_INCREMENTAL=0
    export RUST_LOG=indy=trace
    export RUST_TEST_THREADS=1
    export RUST_BACKTRACE=1
    export OPENSSL_DIR=${OPENSSL_DIR}
    export OPENSSL_LIB_DIR=${OPENSSL_DIR}/lib
    export SODIUM_LIB_DIR=${SODIUM_DIR}/lib
    export SODIUM_INCLUDE_DIR=${SODIUM_DIR}/include
    export LIBZMQ_LIB_DIR=${LIBZMQ_DIR}/lib
    export INDY_LIB_DIR=${INDY_DIR}/lib
    export LIBZMQ_INCLUDE_DIR=${LIBZMQ_DIR}/include
    export TOOLCHAIN_DIR=${TOOLCHAIN_PREFIX}/${TARGET_ARCH}
    export PATH=${TOOLCHAIN_DIR}/bin:${PATH}
    export CC=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang
    export AR=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ar
    export CXX=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang++
    export CXXLD=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ld
    export RANLIB=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ranlib
    export TARGET=android
    export OPENSSL_STATIC=1
}

build_libvcx(){
    echo "**************************************************"
    echo "Building for architecture ${BOLD}${YELLOW}${ABSOLUTE_ARCH}${RESET}"
    echo "Toolchain path ${BOLD}${YELLOW}${TOOLCHAIN_DIR}${RESET}"
    echo "Sodium path ${BOLD}${YELLOW}${SODIUM_DIR}${RESET}"
    echo "Indy path ${BOLD}${YELLOW}${INDY_LIB_DIR}${RESET}"
    echo "Artifacts will be in ${BOLD}${YELLOW}${HOME}/artifacts/${RESET}"
    echo "**************************************************"
    LIBVCX_DIR=$1
    pushd ${LIBVCX_DIR}
        rm -rf target/${TRIPLET}
        cargo clean
        LIBINDY_DIR=${INDY_LIB_DIR} cargo build --release --target=${TRIPLET}
        rm -rf target/${TRIPLET}/release/deps
        rm -rf target/${TRIPLET}/release/build
        rm -rf target/release/deps
        rm -rf target/release/build
    popd
}

copy_libraries_to_jni(){
    JAVA_WRAPPER_DIR=$1
    TARGET_ARCH=$2
    LIBVCX_DIR=$3
    ANDROID_JNI_LIB="${JAVA_WRAPPER_DIR}/android/src/main/jniLibs"
    LIB_PATH=${ANDROID_JNI_LIB}/${ABI}
    echo "Copying dependencies to ${BOLD}${YELLOW}${LIB_PATH}${RESET}"
    mkdir -p $LIB_PATH
    cp ${LIBVCX_DIR}/target/${TRIPLET}/release/{libvcx.a,libvcx.so} ${LIB_PATH}
    cp ${INDY_LIB_DIR}/* ${LIB_PATH}
}
