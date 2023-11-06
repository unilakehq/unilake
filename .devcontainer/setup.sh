#!/bin/bash

set -e

function install_multi_editor_support() {
    echo "Upgrading existing and Installing multi-editor support packages..."
    sudo apt-get update -y
    sudo apt-get upgrade -y
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

    # Define RUSTUP_HOME and CARGO_HOME if they aren't already
    RUSTUP_HOME="/home/coder/.rustup"
    CARGO_HOME="/home/coder/.cargo"

    # Ensure the home directory for coder exists (just in case)
    mkdir -p /home/coder && chown coder:coder /home/coder

    # Download rustup script
    curl -o /tmp/rustup.sh https://sh.rustup.rs
    chmod +x /tmp/rustup.sh

    # Install rust as coder user
    sudo -u coder env RUSTUP_HOME=${RUSTUP_HOME} CARGO_HOME=${CARGO_HOME} /tmp/rustup.sh -y \
        --no-modify-path \
        --profile minimal \
        --default-host x86_64-unknown-linux-gnu

    rm -f /tmp/rustup.sh

    # Add Cargo's bin directory to PATH for current and future sessions
    sudo -u coder echo 'export PATH=$PATH:'${CARGO_HOME}'/bin' >> /home/coder/.bashrc

    # Check installed versions (as coder user for environment consistency)
    sudo -u coder ${CARGO_HOME}/bin/cargo --version
    sudo -u coder ${CARGO_HOME}/bin/rustc --version
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
    echo 'export PATH=$PATH:/root/.pulumi/bin' >> /home/coder/.bashrc
    chown -R coder:coder /root/.pulumi/bin
}

function install_abp_cli() {
    echo "Installing ABP CLI..."
    dotnet tool install -g Volo.Abp.Cli
    echo 'export PATH=$PATH:/root/.dotnet/tools' >> /home/coder/.bashrc

    # wget https://packages.microsoft.com/config/ubuntu/20.04/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
    # dpkg -i packages-microsoft-prod.deb
    # apt-get update && apt-get install -y dotnet-runtime-7.0
}

function setup_docker() {
    echo "Setting up Docker..."
    sudo gpasswd -a coder docker
}

if [[ $EUID -ne 0 ]]; then
   echo "This script must be run with sudo privileges"
   exit 1
fi

export DEBIAN_FRONTEND="noninteractive"

install_multi_editor_support
install_rust
install_dotnet
install_kubectl
install_gh_cli
install_pulumi
install_abp_cli
setup_docker

export PATH=$PATH:/usr/bin/pulumi:/home/coder/.dotnet/tools
chmod o+x /root

echo "Installation completed!"