# Build and publish release of cli
FROM ghcr.io/unilakehq/dev-oss:latest AS build

ARG BUILD_VERSION="0.0.1" 
ARG WORKDIR=/unilake/build
WORKDIR $WORKDIR

COPY unilake-cli ./unilake-cli
COPY unilake-iac ./unilake-iac

USER root
# Install dependencies and utilities
RUN apt-get update && apt-get install -y \
    zip \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir /output

WORKDIR $WORKDIR/unilake-cli/src
RUN dotnet test

WORKDIR $WORKDIR/unilake-cli/src/Unilake.Cli

# Windows
RUN dotnet publish -c release -r win-x64 --self-contained true /p:PublishSingleFile=true /p:DebugType=None /p:Version=$BUILD_VERSION /p:FileVersion=$BUILD_VERSION /p:AssemblyVersion=$BUILD_VERSION
RUN mv bin/release/net7.0/win-x64/publish/Unilake.Cli.exe bin/release/net7.0/win-x64/publish/unilake.exe
RUN zip -j /output/unilake.cli.win-x64.$BUILD_VERSION.zip bin/release/net7.0/win-x64/publish/unilake.exe
RUN cp /output/unilake.cli.win-x64.$BUILD_VERSION.zip /output/unilake.cli.win-x64.latest.zip

# Mac-OS
RUN dotnet publish -c release -r osx-x64 --self-contained true /p:PublishSingleFile=true /p:DebugType=None /p:Version=$BUILD_VERSION /p:FileVersion=$BUILD_VERSION /p:AssemblyVersion=$BUILD_VERSION
RUN mv bin/release/net7.0/osx-x64/publish/Unilake.Cli bin/release/net7.0/osx-x64/publish/unilake
RUN tar -czvf /output/unilake.cli.osx-x64.$BUILD_VERSION.tar.gz -C bin/release/net7.0/osx-x64/publish unilake
RUN cp /output/unilake.cli.osx-x64.$BUILD_VERSION.tar.gz /output/unilake.cli.osx-x64.latest.tar.gz

# Linux
RUN dotnet publish -c release -r linux-x64 --self-contained true /p:PublishSingleFile=true /p:DebugType=None /p:Version=$BUILD_VERSION /p:FileVersion=$BUILD_VERSION /p:AssemblyVersion=$BUILD_VERSION
RUN mv bin/release/net7.0/linux-x64/publish/Unilake.Cli bin/release/net7.0/linux-x64/publish/unilake
RUN tar -czvf /output/unilake.cli.linux-x64.$BUILD_VERSION.tar.gz -C bin/release/net7.0/linux-x64/publish unilake
RUN cp /output/unilake.cli.linux-x64.$BUILD_VERSION.tar.gz /output/unilake.cli.linux-x64.latest.tar.gz

FROM build AS release

RUN curl https://rclone.org/install.sh | bash

ARG RCLONE_CONFIG_TARGETS3_TYPE="s3"
ARG RCLONE_CONFIG_TARGETS3_PROVIDER="Other"
ARG RCLONE_CONFIG_TARGETS3_ACCESS_KEY_ID=""
ARG RCLONE_CONFIG_TARGETS3_SECRET_ACCESS_KEY=""
ARG RCLONE_CONFIG_TARGETS3_REGION=""
ARG RCLONE_CONFIG_TARGETS3_ENDPOINT=""
ARG RCLONE_CONFIG_TARGETS3_LOCATION_CONSTRAINT=""
ARG RCLONE_CONFIG_TARGETS3_ENV_AUTH="false"
ARG RCLONE_CONFIG_TARGETS3_ACL="private"
ARG BUCKET=""

COPY --from=build /output /output
WORKDIR /output

RUN rclone copy ./ targets3:$BUCKET/releases