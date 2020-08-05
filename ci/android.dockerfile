FROM base:1.0.0

USER root

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
        ruby \
        ruby-dev \
        rubygems \
        zip \
    && gem install --no-ri --no-rdoc rake fpm \
    && rm -rf /var/lib/apt/lists/*

# Install JDK
ARG JAVA_VER=8
RUN apt-get update && apt-get install openjdk-${JAVA_VER}-jdk maven -y
ENV JAVA_HOME /usr/lib/jvm/java-${JAVA_VER}-openjdk-amd64

USER indy
# Install Rust toolchain
ARG RUST_VER=1.40.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VER}
ENV PATH /home/indy/.cargo/bin:$PATH

# Set env vars
ARG LIBINDY_VER=1.15.0
ARG LIBVCX_VER=0.8.0
ENV ANDROID_BUILD_FOLDER=/tmp/android_build
ENV ANDROID_SDK=${ANDROID_BUILD_FOLDER}/sdk
ENV ANDROID_SDK_ROOT=${ANDROID_SDK}
ENV ANDROID_HOME=${ANDROID_SDK}
ENV TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/linux
ENV ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r20
ENV PATH=${PATH}:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/tools:${ANDROID_HOME}/tools/bin
ENV LIBINDY_VER=$LIBINDY_VER
ENV LIBVCX_VER=$LIBVCX_VER

COPY --chown=indy:indy ci/scripts/android.prepare.sh .
COPY --chown=indy:indy ci/scripts/setup.android.env.sh .
USER root
RUN chmod +x android.prepare.sh setup.android.env.sh # build_libraries.sh
RUN chown indy:indy android.prepare.sh setup.android.env.sh # build_libraries.sh
USER indy
RUN ./android.prepare.sh

RUN mkdir /home/indy/libvcx-absa
VOLUME ["/home/indy/libvcx-absa"]
