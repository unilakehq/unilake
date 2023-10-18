# SOURCE: https://github.com/coder/enterprise-images/tree/main/images
FROM codercom/enterprise-base:ubuntu

# Run everything as root
USER root

# Packages required for multi-editor support
RUN DEBIAN_FRONTEND="noninteractive" apt-get update -y && \
    apt-get install -y \
    libxtst6 \
    libxrender1 \
    libfontconfig1 \
    libxi6 \
    libgtk-3-0

#### ---> INSTALL Rust

# Set environment variables 
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=/opt/cargo/bin:$PATH

# Install Rust
COPY rustup.sh /tmp/rustup.sh
RUN chmod +x /tmp/rustup.sh && /tmp/rustup.sh -y \
                   --no-modify-path \
                   --profile minimal \
                   --default-toolchain stable \
                   --default-host x86_64-unknown-linux-gnu && \
    rm -f /tmp/rustup.sh && \
    chmod -R a+w ${RUSTUP_HOME} ${CARGO_HOME}

# Validate that cargo and rustc are available
RUN cargo --version && rustc --version

#### ---> INSTALL Node
# Install whichever Node version is LTS
# RUN curl -sL https://deb.nodesource.com/setup_lts.x | bash -
# RUN DEBIAN_FRONTEND="noninteractive" apt-get update -y && \
#   apt-get install -y nodejs

# Install Yarn
# RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -
# RUN echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list
# RUN DEBIAN_FRONTEND="noninteractive" apt-get update && apt-get install -y yarn

#### ---> INSTALL Dotnet
# See: https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-install-script
RUN wget "https://dot.net/v1/dotnet-install.sh" \
    && chmod +x dotnet-install.sh \
    && ./dotnet-install.sh --install-dir /workspace/dotnet --channel STS \
    && rm dotnet-install.sh \
    && sudo ln -s /workspace/dotnet/dotnet /usr/bin/dotnet

#### ---> INSTALL Kubectl
RUN curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

#### ---> INSTALL gh cli
RUN type -p curl >/dev/null || sudo apt install curl -y && \
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && sudo chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && sudo apt update \
    && sudo apt install gh -y

#### ---> INSTALL pulumi
RUN curl -fsSL https://get.pulumi.com | sh
ENV PATH $PATH:/usr/bin/pulumi

#### ---> Abp Cli
RUN dotnet tool install -g Volo.Abp.Cli
ENV PATH $PATH:/home/coder/.dotnet/tools

# Set back to coder user
USER coder