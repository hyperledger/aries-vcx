FROM ubuntu:18.04

ARG uid=1000

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
      software-properties-common

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
    && gem install --no-ri --no-rdoc rake fpm \
    && rm -rf /var/lib/apt/lists/*

# install java and maven
RUN apt-get update && apt-get install openjdk-8-jdk -y
ENV JAVA_HOME /usr/lib/jvm/java-8-openjdk-amd64
RUN apt-get update && apt-get install -y maven

# install nodejs and npm
RUN curl -sL https://deb.nodesource.com/setup_10.x | bash -
RUN apt-get install -y nodejs

# Install .NET CLI dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libc6 \
        libgcc1 \
        libgssapi-krb5-2 \
        libicu60 \
        libssl1.1 \
        libstdc++6 \
        zlib1g \
    && rm -rf /var/lib/apt/lists/*

# Install .NET Core SDK
RUN dotnet_sdk_version=3.1.404 \
    && curl -SL --output dotnet.tar.gz https://dotnetcli.azureedge.net/dotnet/Sdk/$dotnet_sdk_version/dotnet-sdk-$dotnet_sdk_version-linux-x64.tar.gz \
    && dotnet_sha512='94d8eca3b4e2e6c36135794330ab196c621aee8392c2545a19a991222e804027f300d8efd152e9e4893c4c610d6be8eef195e30e6f6675285755df1ea49d3605' \
    && echo "$dotnet_sha512 dotnet.tar.gz" | sha512sum -c - \
    && mkdir -p /usr/share/dotnet \
    && tar -ozxf dotnet.tar.gz -C /usr/share/dotnet \
    && rm dotnet.tar.gz \
    && ln -s /usr/share/dotnet/dotnet /usr/bin/dotnet \
    # Trigger first run experience by running arbitrary cmd
    && dotnet help

# Install Mono
RUN apt-get install gnupg ca-certificates \
    && apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF \
    && add-apt-repository "deb https://download.mono-project.com/repo/ubuntu stable-bionic main" \
    && apt-get update \
    && apt-get install -y mono-devel

# workaround to a bug in the dotnet version being used
RUN cd /usr/share/dotnet/sdk/3.1.404/Sdks/Microsoft.NET.Sdk.WindowsDesktop/targets \
    && mv Microsoft.WinFx.props Microsoft.WinFX.props \
    && mv Microsoft.WinFx.targets Microsoft.WinFX.targets \
    && cd ~

RUN apt-get install -y wget

RUN useradd -ms /bin/bash -u $uid indy
USER indy

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.58.0
ENV PATH /home/indy/.cargo/bin:$PATH

RUN cargo install cargo-deb --no-default-features

WORKDIR /home/indy
