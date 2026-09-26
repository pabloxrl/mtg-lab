#!/usr/bin/env bash
set -euo pipefail
case "$TARGETARCH" in
 arm64) arch=aarch64; symarch=arm64; codex_sha=499fe70d70f4e4904b6a5a4ec1b1edf6c4a1a47a075ea7e2ec2b5262ba47b471; sym_sha=bf204639b836d378f5d2ddb389f5d76602a62e9d1b115bdbefb07916848ae066 ;;
 amd64) arch=x86_64; symarch=x86_64; codex_sha=0e211868c9fd73cb49ad35ac675b5eafdf6b9f453df8a493df980c59a590fe5f; sym_sha=ea35a04a54a6d37c0cafe3f195da871e47614a8c05765b90dbb4cac32e1435ee ;;
 *) echo "Unsupported architecture" >&2; exit 1 ;;
esac
curl --fail --location --retry 3 https://github.com/openai/codex/releases/download/rust-v0.157.1/codex-package-${arch}-unknown-linux-musl.tar.gz -o /tmp/codex.tgz
echo "$codex_sha  /tmp/codex.tgz" | sha256sum -c -
mkdir /opt/codex
tar -xzf /tmp/codex.tgz -C /opt/codex
rm /tmp/codex.tgz
curl --fail --location --retry 3 https://github.com/openai/symphony/releases/download/v0.0.3/symphony-v0.0.3-linux_${symarch} -o /usr/local/bin/symphony
echo "$sym_sha  /usr/local/bin/symphony" | sha256sum -c -
chmod +x /usr/local/bin/symphony
