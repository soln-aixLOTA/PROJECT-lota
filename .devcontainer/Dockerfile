FROM mcr.microsoft.com/vscode/devcontainers/rust:latest

# Install Docker
RUN curl -fsSL https://get.docker.com -o get-docker.sh \
    && sh get-docker.sh \
    && rm get-docker.sh

# Add vscode user to docker group
RUN usermod -aG docker vscode 