FROM ghcr.io/unilakehq/dev-oss:latest AS build

USER root

ARG BUILD_VERSION="0.0.1"
ARG WORKDIR=/unilake/build

WORKDIR $WORKDIR

COPY . ./

WORKDIR $WORKDIR/src
RUN dotnet build -c Release -p:Version=$BUILD_VERSION -p:DebugType=None -p:DebugSymbols=false
RUN dotnet pack --configuration Release

FROM build AS publish

USER root

ARG GITHUB_TOKEN=""

ARG WORKDIR=/release
WORKDIR $WORKDIR

COPY --from=build /unilake/build/src/Unilake.Iac/bin/Release .

RUN dotnet nuget push ./Unilake.Iac.$BUILD_VERSION.nupkg --api-key $GITHUB_TOKEN --source "github"