# SOURCE: https://github.com/coder/enterprise-images/tree/main/images
FROM codercom/enterprise-base:ubuntu

# Run everything as root
USER root

# Load and run setup script
COPY setup.sh /tmp/setup.sh
RUN chmod +x /tmp/setup.sh && /tmp/setup.sh

# Set back to coder user
USER coder
RUN /bin/bash -c "source ~/.bashrc"
