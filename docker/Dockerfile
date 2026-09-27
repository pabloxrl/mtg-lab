FROM rust:1.98.1-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS rust
RUN rustup component add rustfmt clippy
FROM maven:3.9.11-eclipse-temurin-21-noble@sha256:6fdc855a6ed81d288ca7ca37ac6ff5e9308b612485c0801d70b25a858c83d237 AS toolchain
USER root
RUN apt-get update && apt-get install -y --no-install-recommends python3 git gh curl ca-certificates build-essential pkg-config libssl-dev ripgrep bubblewrap tini && rm -rf /var/lib/apt/lists/*
COPY --from=rust /usr/local/rustup /opt/rustup
COPY --from=rust /usr/local/cargo /opt/cargo
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/home/agent/.cargo PATH=/opt/cargo/bin:/opt/codex/bin:${PATH}
ARG TARGETARCH
COPY docker/install-tools.sh /tmp/install-tools.sh
RUN bash /tmp/install-tools.sh && rm /tmp/install-tools.sh
RUN useradd --create-home --uid 1001 --shell /bin/bash agent && mkdir -p /home/agent/.cargo /home/agent/.codex /home/agent/.local/share/mtg-lab-symphony /home/agent/.cache && chown -R agent:agent /home/agent
ENV HOME=/home/agent SYMPHONY_PYTHON=/usr/bin/python3 SYMPHONY_CODEX=/opt/codex/bin/codex SYMPHONY_CONTROL_ROOT=/opt/mtg-lab MTG_CONTAINER=1
USER agent
WORKDIR /home/agent
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["bash"]
FROM toolchain AS runtime
USER root
RUN apt-get update && apt-get install -y --no-install-recommends nginx nodejs && rm -rf /var/lib/apt/lists/*
USER agent
COPY --chown=root:root . /opt/mtg-lab
CMD ["python3", "/opt/mtg-lab/scripts/symphony/container_entrypoint.py"]
