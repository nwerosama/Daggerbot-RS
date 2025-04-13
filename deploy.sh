#!/bin/bash

MACHINE_USER=toast
MACHINE_IP=192.168.68.100
DOCKER_REG=ghcr.io/nwerosama/daggerbot-rs
DOCKER_TAG=koi
SSH_EXIT_STATUS=$?

ssh $MACHINE_USER@$MACHINE_IP "script -qc 'docker service update daggerbot_app --force --image $DOCKER_REG:$DOCKER_TAG --with-registry-auth' /dev/null"
# ssh $MACHINE_USER@$MACHINE_IP "script -qc 'docker service scale daggerbot_app=0' /dev/null"

if [ $SSH_EXIT_STATUS -eq 0 ]; then
  for SWARM_HOST in 192.168.68.100 192.168.68.101 192.168.68.102; do
    ssh $MACHINE_USER@$SWARM_HOST "docker system prune -f"
  done
fi
