FROM ubuntu:18.04

ARG USER_ID=1000

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config \
      libgmp3-dev \
      curl \
      wget \
      build-essential \
      libsqlite3-dev \
      cmake \
      apt-transport-https \
      ca-certificates \
      debhelper \
      devscripts \
      libzmq3-dev \
      zip \
      unzip \
      python3-distutils \
      jq

# Add indy user to sudoers
RUN useradd -ms /bin/bash -u $USER_ID indy
RUN usermod -aG sudo indy

USER root

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
        ruby \
        ruby-dev \
        rubygems \
    && gem install --no-ri --no-rdoc rake fpm \
    && rm -rf /var/lib/apt/lists/*

# Install JDK
ARG JAVA_VER=8
RUN apt-get update && apt-get install openjdk-${JAVA_VER}-jdk maven -y
ENV JAVA_HOME /usr/lib/jvm/java-${JAVA_VER}-openjdk-amd64

WORKDIR /home/indy

# Install node
ARG NODE_VER=8.x
RUN curl -sL https://deb.nodesource.com/setup_${NODE_VER} | bash -
RUN apt-get install -y nodejs

USER indy

# Install Rust toolchain
ARG RUST_VER=1.55.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VER}
ENV PATH /home/indy/.cargo/bin:$PATH


# This is to mount a host volume with write access
RUN mkdir /home/indy/aries-vcx
# VOLUME ["/home/indy/aries-vcx"]

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
ENV LIBVCX_VERSION=$LIBVCX_VER

# COPY --chown=indy:indy ci/scripts/android.prepare.sh .
# COPY --chown=indy:indy ci/scripts/setup.android.env.sh .
# RUN chmod +x android.prepare.sh setup.android.env.sh

# This is temporary workaround GA mounted directory issues
COPY --chown=indy:indy . aries-vcx/

RUN ./aries-vcx/wrappers/java/ci/android.prepare.sh
