FROM ubuntu:20.04

ARG USER_ID=1000
ARG DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update -y -qq && \
    apt-get install -y --no-install-recommends \
      pkg-config \
      curl \
      wget \
      build-essential \
      perl \
      cmake \
      zip \
      unzip \
      python3-distutils \
      jq \
      ruby \
      ruby-dev \
      rubygems \
    && gem install -no-ri-no-doc rake \
    && rm -rf /var/lib/apt/lists/*

# Install JDK
ARG JAVA_VER=8
RUN apt-get update && apt-get install openjdk-${JAVA_VER}-jdk maven -y
ENV JAVA_HOME /usr/lib/jvm/java-${JAVA_VER}-openjdk-amd64

# Add indy user to sudoers
RUN useradd -ms /bin/bash -u $USER_ID indy
RUN usermod -aG sudo indy

WORKDIR /home/indy

# Install node
ARG NODE_VER=12.x
RUN curl -sL https://deb.nodesource.com/setup_${NODE_VER} | bash -
RUN apt-get install -y nodejs

USER indy

# Install Rust toolchain
ARG RUST_VER=1.64.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VER}
ENV PATH /home/indy/.cargo/bin:$PATH

RUN mkdir -p /home/indy/aries-vcx/wrappers/java/ci/

# Set env vars
ARG LIBVCX_VER=0.8.0
ENV ANDROID_BUILD_FOLDER=/tmp/android_build
ENV ANDROID_SDK=${ANDROID_BUILD_FOLDER}/sdk
ENV ANDROID_SDK_ROOT=${ANDROID_SDK}
ENV ANDROID_HOME=${ANDROID_SDK}
ENV TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/linux
ENV ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r20
ENV PATH=${PATH}:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/tools:${ANDROID_HOME}/tools/bin
ENV LIBVCX_VERSION=$LIBVCX_VER

COPY ./wrappers/java/ci/ aries-vcx/wrappers/java/ci/

RUN ./aries-vcx/wrappers/java/ci/android.prepare.sh

COPY --chown=indy:indy . aries-vcx/
