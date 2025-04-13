#!/bin/bash

ENV_FILE=.env.bot

export DOCKER_HOSTNAME=$(hostname)
export $(grep -v '^#' $ENV_FILE | xargs)
clear && cargo fmt && cargo run daggerbotbeta
unset DOCKER_HOSTNAME
unset $(grep -v '^#' $ENV_FILE | cut -d= -f1)
