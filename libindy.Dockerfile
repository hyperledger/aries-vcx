FROM ubuntu:16.04 as BASE

ARG uid=1000

RUN apt-get update && \
    apt-get install -y \
      pkg-config \
      libssl-dev \
      libgmp3-dev \
      curl \
      build-essential \
      libsqlite3-dev \
      cmake \
      git \
      python3.5 \
      python3-pip \
      python-setuptools \
      apt-transport-https \
      ca-certificates \
      debhelper \
      wget \
      devscripts \
      libncursesw5-dev \
      libzmq3-dev \
      zip \
      unzip \
      jq


RUN pip3 install -U \
	pip \
	setuptools \
	virtualenv \
	twine \
	plumbum \
	deb-pkg-tools

RUN cd /tmp && \
   curl https://download.libsodium.org/libsodium/releases/libsodium-1.0.18.tar.gz | tar -xz && \
    cd /tmp/libsodium-1.0.18 && \
    ./configure --disable-shared && \
    make && \
    make install && \
    rm -rf /tmp/libsodium-1.0.18

RUN useradd -ms /bin/bash -u $uid indy
USER indy

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.43.1
ENV PATH /home/indy/.cargo/bin:$PATH

ARG INDYSDK_REVISION
ARG INDYSDK_REPO
WORKDIR /home/indy
RUN git clone "${INDYSDK_REPO}" "./indy-sdk"
RUN cd "/home/indy/indy-sdk" && git checkout "${INDYSDK_REVISION}"

RUN cargo build --release --manifest-path=/home/indy/indy-sdk/libindy/Cargo.toml
USER root
RUN mv /home/indy/indy-sdk/libindy/target/release/*.so /usr/lib
USER indy
RUN cargo build --release --manifest-path=/home/indy/indy-sdk/experimental/plugins/postgres_storage/Cargo.toml
USER root
RUN mv /home/indy/indy-sdk/experimental/plugins/postgres_storage/target/release/*.so /usr/lib

FROM ubuntu:16.04

RUN apt-get update && \
    apt-get install -y \
      libssl-dev \
      apt-transport-https \
      ca-certificates

RUN useradd -ms /bin/bash -u 1000 indy
USER indy

WORKDIR /home/indy

COPY --from=BASE /var/lib/dpkg/info /var/lib/dpkg/info
COPY --from=BASE /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
COPY --from=BASE /usr/local /usr/local

COPY --from=BASE /usr/lib/libindy.so /usr/lib/libindy.so
COPY --from=BASE /usr/lib/libindystrgpostgres.so /usr/lib/libindystrgpostgres.so

LABEL org.label-schema.schema-version="1.0"
LABEL org.label-schema.name="indysdk"
LABEL org.label-schema.version="${INDY_VERSION}"

USER indy
