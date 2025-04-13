#!/bin/bash

REGISTRY=ghcr.io/nwerosama/daggerbot-rs
TAG_NAME=$(git rev-parse --abbrev-ref HEAD)
COMMIT=$(git rev-parse --short HEAD)
echo "Building on $TAG_NAME branch with commit hash $COMMIT"

export GIT_COMMIT_HASH=$(git rev-parse HEAD) && \
cargo zigbuild --target x86_64-unknown-linux-musl --locked -rF production && \
docker build -t $REGISTRY:$TAG_NAME . && docker push $REGISTRY:$TAG_NAME
