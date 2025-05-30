#!/bin/bash

REGISTRY=ghcr.io/nwerosama/daggerbot-rs
TAG_NAME=$(git rev-parse --abbrev-ref HEAD)
COMMIT=$(git rev-parse --short HEAD)
echo "Building on $TAG_NAME branch with commit hash $COMMIT"

cargo build --locked -rF production && \
docker build -t $REGISTRY:$TAG_NAME . && docker push $REGISTRY:$TAG_NAME
