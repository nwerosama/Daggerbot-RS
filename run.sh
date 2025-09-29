#!/bin/bash

ENV_FILE=.env

export DOCKER_HOSTNAME=$(hostname)
export $(grep -v '^#' $ENV_FILE | xargs)
clear && cargo fmt && RUST_LOG=debug cargo run daggerbotbeta
unset DOCKER_HOSTNAME
unset $(grep -v '^#' $ENV_FILE | cut -d= -f1)
