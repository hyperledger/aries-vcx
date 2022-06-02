FROM ubuntu:20.04

ARG uid=1000

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update && \
    apt-get install -y \
      pkg-config \
      libssl-dev \
      curl \
      ca-certificates \
      libsqlite3-dev \
      cmake \
      python3-pip \
      debhelper \
      devscripts \
      libncursesw5-dev \
      libzmq3-dev \
      libsodium-dev \
      software-properties-common \
      wget

# Adding Evernym ca cert
RUN mkdir -p /usr/local/share/ca-certificates
RUN curl -k https://repo.corp.evernym.com/ca.crt | tee /usr/local/share/ca-certificates/Evernym_Root_CA.crt
RUN update-ca-certificates

RUN pip3 install --upgrade pip==21.0.1

RUN pip3 install -U \
	pip \
	twine \
	plumbum==1.6.7 six==1.12.0 \
	deb-pkg-tools

RUN apt-get update && apt-get install -y --no-install-recommends \
        ruby \
        ruby-dev \
        rubygems \
    && gem install --no-document rake fpm \
    && rm -rf /var/lib/apt/lists/*

# install java and maven
RUN apt-get update && apt-get install openjdk-8-jdk -y
ENV JAVA_HOME /usr/lib/jvm/java-8-openjdk-amd64
RUN apt-get update && apt-get install -y maven

# install nodejs and npm
RUN curl -sL https://deb.nodesource.com/setup_12.x | bash -
RUN apt-get install -y nodejs

# Install .NET Core SDK repo
RUN wget https://packages.microsoft.com/config/ubuntu/20.04/packages-microsoft-prod.deb \
    && dpkg -i packages-microsoft-prod.deb

# Install Mono and .NET Core SDK
RUN apt-get install gnupg ca-certificates \
    && apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF \
    && add-apt-repository "deb https://download.mono-project.com/repo/ubuntu stable-bionic main" \
    && apt-get update \
    && apt-get install -y dotnet-sdk-3.1 mono-devel

RUN apt-get install -y wget

RUN useradd -ms /bin/bash -u $uid indy
USER indy

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.61.0
ENV PATH /home/indy/.cargo/bin:$PATH

RUN cargo install cargo-deb

EXPOSE 8080

WORKDIR /home/indy
