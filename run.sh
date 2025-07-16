#!/bin/bash

ENV_FILE=.env.bot

export DOCKER_HOSTNAME=$(hostname)
export $(grep -v '^#' $ENV_FILE | xargs)
clear && cargo fmt && RUST_LOG=debug cargo run daggerbotbeta
# clear && gdb -return-child-result -batch -ex run -ex thread apply all bt -ex quit --args target/debug/daggerbot daggerbotbeta
unset DOCKER_HOSTNAME
unset $(grep -v '^#' $ENV_FILE | cut -d= -f1)
