# SOURCE: https://github.com/coder/enterprise-images/tree/main/images
FROM codercom/enterprise-base:ubuntu

# Run everything as root
USER root

# Set the environment for non-interactive installations
ENV DEBIAN_FRONTEND=noninteractive

# Update and upgrade the existing packages
RUN apt-get update -y \
    && apt-get upgrade -y \
    # Install dependencies
    && apt-get install -y \
    libxtst6 \
    libxrender1 \
    libfontconfig1 \
    libxi6 \
    nano \
    iproute2 \
    libgtk-3-0 \
    wget \
    curl \
    ca-certificates \
    gnupg \
    tar \
    # Clean up
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && NODE_MAJOR=21 \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install nodejs -y

# Install Rust
ENV RUSTUP_HOME="/home/coder/.rustup" \
    CARGO_HOME="/home/coder/.cargo"
RUN mkdir -p /home/coder && chown coder:coder /home/coder \
    && curl -o /tmp/rustup.sh https://sh.rustup.rs \
    && chmod +x /tmp/rustup.sh \
    && sudo -u coder /tmp/rustup.sh -y --no-modify-path --profile minimal --default-host x86_64-unknown-linux-gnu \
    && rm /tmp/rustup.sh

# Install Dotnet
ENV DOTNET_ROOT="/workspace/dotnet"
ENV PATH="${PATH}:${DOTNET_ROOT}"
RUN wget -qO- "https://dot.net/v1/dotnet-install.sh" | bash -s -- --install-dir ${DOTNET_ROOT} -Channel 7.0 \
    && ln -s ${DOTNET_ROOT}/dotnet /usr/local/bin/dotnet \
    # Check to ensure the 'dotnet' command is available
    && dotnet --version

# Install Kubectl
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" \
    && install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl \
    && rm kubectl

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list \
    && apt update \
    && apt install gh -y

# Install Pulumi
RUN curl -fsSL https://get.pulumi.com | sudo -u coder sh

# Install ABP CLI (Dotnet Global Tool)
RUN sudo -u coder dotnet tool install -g Volo.Abp.Cli

# Install k3d
RUN curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | sudo -u coder bash

# Install Helm
RUN curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | sudo -u coder bash

# Setup Docker
RUN gpasswd -a coder docker \
    && chown coder /var/run/docker.sock || true

# Setup KubeFwd
ARG KUBEFWD_URL=https://github.com/txn2/kubefwd/releases/download/1.22.5/kubefwd_Linux_x86_64.tar.gz
RUN mkdir -p /opt/kubefwd && \
    curl -L https://github.com/txn2/kubefwd/releases/download/1.22.5/kubefwd_Linux_x86_64.tar.gz | tar xz -C /opt/kubefwd

# Setup Unilake CLI
RUN curl -L https://unilake-cli.s3.nl-ams.scw.cloud/releases/unilake.cli.linux-x64.latest.tar.gz | tar xzf - && \
    chmod +x unilake && \
    mv unilake /usr/local/bin/ && \
    rm -f unilake.cli.linux-x64.latest.tar.gz

# Set back to coder user
USER coder

# Set the PATH explicitly for the tools installed
ENV PATH="${PATH}:${CARGO_HOME}/bin:/home/coder/.pulumi/bin:/home/coder/.dotnet/tools:/opt/kubefwd:"${CARGO_HOME}"/bin"

# Final clean up
RUN sudo chmod o+x /root