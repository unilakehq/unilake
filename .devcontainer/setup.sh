#!/bin/bash

set -e

function install_multi_editor_support() {
    echo "Installing multi-editor support packages..."
    sudo apt-get update -y
    sudo apt-get install -y \
    libxtst6 \
    libxrender1 \
    libfontconfig1 \
    libxi6 \
    nano \
    libgtk-3-0
}

function install_rust() {
    echo "Installing Rust..."
    curl -o /tmp/rustup.sh https://sh.rustup.rs
    chmod +x /tmp/rustup.sh
    /tmp/rustup.sh -y \
    --no-modify-path \
    --profile minimal \
    --default-toolchain stable \
    --default-host x86_64-unknown-linux-gnu
    rm -f /tmp/rustup.sh
    chmod -R a+w ${RUSTUP_HOME} ${CARGO_HOME}
    cargo --version
    rustc --version
}

function install_dotnet() {
    echo "Installing Dotnet..."
    wget "https://dot.net/v1/dotnet-install.sh"
    chmod +x dotnet-install.sh
    ./dotnet-install.sh --install-dir /workspace/dotnet --channel STS
    rm dotnet-install.sh
    sudo ln -s /workspace/dotnet/dotnet /usr/bin/dotnet
}

function install_kubectl() {
    echo "Installing Kubectl..."
    curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
    sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
    rm kubectl
}

function install_gh_cli() {
    echo "Installing GitHub CLI..."
    type -p curl >/dev/null || sudo apt install curl -y
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
    sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
    sudo apt update
    sudo apt install gh -y
}

function install_pulumi() {
    echo "Installing Pulumi..."
    curl -fsSL https://get.pulumi.com | sh
}

function install_abp_cli() {
    echo "Installing ABP CLI..."
    dotnet tool install -g Volo.Abp.Cli
}

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run with sudo privileges"
   exit 1
fi

export DEBIAN_FRONTEND="noninteractive"
export RUSTUP_HOME=/opt/rustup
export CARGO_HOME=/opt/cargo
export PATH=/opt/cargo/bin:$PATH

install_multi_editor_support
install_rust
install_dotnet
install_kubectl
install_gh_cli
install_pulumi
install_abp_cli

export PATH=$PATH:/usr/bin/pulumi:/home/coder/.dotnet/tools

echo "Installation completed!"